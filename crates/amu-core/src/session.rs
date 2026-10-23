use crate::{
    ComposerError, DecisionStatus, Language,
    context::RecentContext,
    display_policy::DisplayPolicy,
    korean_buffer::KoreanBuffer,
    korean_initials,
    language::{Decoder, Reading, SegmentOrder},
    selection::Decision,
};

const MAX_TOKEN_BYTES: usize = 192;
const MAX_LITERAL_KEYS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Auto,
    Korean,
    English,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Letter(u8),
    Backspace,
    Finish,
    WordBoundary,
    CycleReading,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Update {
    pub consumed: bool,
    pub commit: String,
    pub preedit: String,
    pub language: Language,
}

#[derive(Debug)]
pub struct Interpretation {
    pub status: DecisionStatus,
    pub alternative: Option<Language>,
    pub margin: Option<f32>,
}

pub struct Session {
    korean: KoreanBuffer,
    mixed_korean: KoreanBuffer,
    decoder: Decoder,
    context: RecentContext,
    literal: String,
    mode: Mode,
    decision: Decision,
    edited: bool,
    manual: bool,
    literal_complete: bool,
}

impl Session {
    pub fn new(mode: Mode) -> Result<Self, ComposerError> {
        Ok(Self {
            korean: KoreanBuffer::new()?,
            mixed_korean: KoreanBuffer::new()?,
            decoder: Decoder::new()?,
            context: RecentContext::default(),
            literal: String::with_capacity(MAX_LITERAL_KEYS),
            mode,
            decision: match mode {
                Mode::Auto => Decision::empty(),
                Mode::Korean => Decision::fixed(Reading::Korean),
                Mode::English => Decision::fixed(Reading::English),
            },
            edited: false,
            manual: false,
            literal_complete: true,
        })
    }

    pub fn process(&mut self, input: Input) -> Update {
        match input {
            Input::Letter(ascii) if ascii.is_ascii_alphabetic() => self.append(ascii),
            Input::Letter(_) | Input::Finish => self.finish(),
            Input::Backspace => self.backspace(),
            Input::WordBoundary => self.commit_word(),
            Input::CycleReading => self.cycle_reading(),
        }
    }

    /// Inspect selection evidence without retaining or logging user text.
    pub fn interpretation(&self) -> Interpretation {
        Interpretation {
            status: self.decision.status,
            alternative: self.decision.alternative.map(Reading::language),
            margin: self.decision.margin,
        }
    }

    pub fn change_mode(&mut self, mode: Mode) -> Update {
        let update = self.finish();
        self.mode = mode;
        self.select_reading();
        update
    }

    pub fn reset(&mut self) {
        self.context.clear();
        self.clear_composition();
    }

    fn clear_composition(&mut self) {
        self.korean.reset();
        self.mixed_korean.reset();
        self.decoder.reset();
        self.literal.clear();
        self.edited = false;
        self.manual = false;
        self.literal_complete = true;
        self.select_reading();
    }

    fn append(&mut self, ascii: u8) -> Update {
        let mut commit = String::new();
        if self.literal.len() >= MAX_LITERAL_KEYS
            || self.korean.byte_len() >= MAX_TOKEN_BYTES
            || self.mixed_korean.byte_len() >= MAX_TOKEN_BYTES
        {
            commit = self.finish().commit;
        }
        self.literal.push(ascii as char);
        if self.decision.reading == Reading::Initials {
            // Initials retain literal keys, including after deletion. A later
            // vowel can resume normal composition without losing a consonant.
            self.edited = false;
            if self.manual && !korean_initials::can_render(&self.literal) {
                self.korean.continue_initials(&self.literal);
                self.decision = Decision::fixed(Reading::Korean);
                self.literal_complete = false;
            } else {
                self.korean.recompose(&self.literal);
            }
        } else if self.mode != Mode::English
            && (!self.edited || self.decision.reading == Reading::Korean)
        {
            self.korean.append(ascii);
        } else if self.edited
            && matches!(
                self.decision.reading,
                Reading::Mixed {
                    order: SegmentOrder::EnglishFirst,
                    ..
                }
            )
        {
            self.mixed_korean.append(ascii);
        }
        if self.manual && !self.edited {
            self.restore_mixed_reading();
        } else if !self.edited {
            self.select_reading();
        }
        self.update(true, commit)
    }

    fn backspace(&mut self) -> Update {
        let consumed = match self.decision.reading {
            Reading::English | Reading::Initials => {
                self.korean.reset();
                self.literal.pop().is_some()
            }
            Reading::Korean => {
                self.literal.clear();
                self.literal_complete = false;
                self.korean.backspace()
            }
            Reading::Mixed {
                boundary,
                order: SegmentOrder::KoreanFirst,
            } => {
                self.literal.pop();
                if self.literal.len() == boundary {
                    // Resume the prefix's actual libhangul state, not a shorter
                    // replay of the full token that could merge syllables.
                    std::mem::swap(&mut self.korean, &mut self.mixed_korean);
                    self.mixed_korean.reset();
                    self.literal.clear();
                    self.literal_complete = false;
                    self.decision.reading = Reading::Korean;
                }
                true
            }
            Reading::Mixed {
                boundary,
                order: SegmentOrder::EnglishFirst,
            } => {
                let consumed = self.mixed_korean.backspace();
                self.literal.truncate(boundary);
                self.literal_complete = false;
                if self.mixed_korean.byte_len() == 0 {
                    self.korean.reset();
                    self.literal_complete = true;
                    self.decision.reading = Reading::English;
                }
                consumed
            }
        };
        // Editing freezes the current interpretation. Replaying fewer raw keys would
        // erase syllable boundaries (갑사 + Backspace must become 갑ㅅ, not 값).
        self.edited = true;
        self.decision.status = DecisionStatus::Edited;
        self.decision.alternative = None;
        self.decision.margin = None;
        if !consumed {
            self.reset();
        } else if self.literal.is_empty() && self.korean.byte_len() == 0 {
            self.clear_composition();
        }
        self.update(consumed, String::new())
    }

    fn cycle_reading(&mut self) -> Update {
        // After a Korean deletion the original keys no longer describe the
        // displayed syllable boundaries. Keep that edited composition intact.
        if self.mode != Mode::Auto || !self.literal_complete || self.literal.is_empty() {
            return self.update(true, String::new());
        }
        self.korean.recompose(&self.literal);
        let readings = self
            .decoder
            .manual_readings(&self.literal, &self.korean, &self.context);
        let next = readings
            .iter()
            .position(|reading| *reading == self.decision.reading)
            .map_or(0, |index| (index + 1) % readings.len());
        self.decision = Decision::fixed(readings[next]);
        self.manual = true;
        self.edited = false;
        self.restore_mixed_reading();
        self.update(true, String::new())
    }

    fn finish(&mut self) -> Update {
        self.context.clear();
        self.commit()
    }

    fn commit_word(&mut self) -> Update {
        let supported = matches!(
            self.decision.status,
            DecisionStatus::Supported | DecisionStatus::Explicit
        );
        let update = self.commit();
        if !update.commit.is_empty() {
            self.context.observe(supported.then_some(update.language));
        }
        update
    }

    fn commit(&mut self) -> Update {
        let language = self.decision.reading.language();
        let commit = self.preedit();
        self.clear_composition();
        Update {
            consumed: false,
            commit,
            preedit: String::new(),
            language,
        }
    }

    fn update(&self, consumed: bool, commit: String) -> Update {
        Update {
            consumed,
            commit,
            preedit: self.preedit(),
            language: self.decision.reading.language(),
        }
    }

    fn select_reading(&mut self) {
        self.decision = match self.mode {
            Mode::Korean => Decision::fixed(Reading::Korean),
            Mode::English => Decision::fixed(Reading::English),
            Mode::Auto => self.decoder.decide(
                &self.literal,
                &self.korean,
                &self.context,
                DisplayPolicy::after(self.decision.reading, self.literal.len().saturating_sub(1)),
            ),
        };
        self.restore_mixed_reading();
    }

    fn restore_mixed_reading(&mut self) {
        if let Reading::Mixed { boundary, order } = self.decision.reading {
            let korean_keys = match order {
                SegmentOrder::KoreanFirst => &self.literal[..boundary],
                SegmentOrder::EnglishFirst => &self.literal[boundary..],
            };
            self.mixed_korean.recompose(korean_keys);
        }
    }

    fn preedit(&self) -> String {
        match self.decision.reading {
            Reading::English => self.literal.clone(),
            Reading::Korean => self.korean.text(),
            Reading::Initials => korean_initials::render(&self.literal),
            Reading::Mixed {
                boundary,
                order: SegmentOrder::KoreanFirst,
            } => {
                format!("{}{}", self.mixed_korean.text(), &self.literal[boundary..])
            }
            Reading::Mixed {
                boundary,
                order: SegmentOrder::EnglishFirst,
            } => {
                format!("{}{}", &self.literal[..boundary], self.mixed_korean.text())
            }
        }
    }
}
