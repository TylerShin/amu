use crate::{
    ComposerError, Language,
    context::RecentContext,
    korean_buffer::KoreanBuffer,
    korean_initials,
    language::{Decoder, Reading, SegmentOrder},
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
}

#[derive(Debug, PartialEq, Eq)]
pub struct Update {
    pub consumed: bool,
    pub commit: String,
    pub preedit: String,
    pub language: Language,
}

pub struct Session {
    korean: KoreanBuffer,
    mixed_korean: KoreanBuffer,
    decoder: Decoder,
    context: RecentContext,
    literal: String,
    mode: Mode,
    reading: Reading,
    edited: bool,
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
            reading: if mode == Mode::Korean {
                Reading::Korean
            } else {
                Reading::English
            },
            edited: false,
        })
    }

    pub fn process(&mut self, input: Input) -> Update {
        match input {
            Input::Letter(ascii) if ascii.is_ascii_alphabetic() => self.append(ascii),
            Input::Letter(_) | Input::Finish => self.finish(),
            Input::Backspace => self.backspace(),
            Input::WordBoundary => self.commit_word(),
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
        if self.reading == Reading::Initials {
            // Initials retain literal keys, including after deletion. A later
            // vowel can resume normal composition without losing a consonant.
            self.korean.recompose(&self.literal);
            self.edited = false;
        } else if self.mode != Mode::English && (!self.edited || self.reading == Reading::Korean) {
            self.korean.append(ascii);
        } else if self.edited
            && matches!(
                self.reading,
                Reading::Mixed {
                    order: SegmentOrder::EnglishFirst,
                    ..
                }
            )
        {
            self.mixed_korean.append(ascii);
        }
        if !self.edited {
            self.select_reading();
        }
        self.update(true, commit)
    }

    fn backspace(&mut self) -> Update {
        let consumed = match self.reading {
            Reading::English | Reading::Initials => {
                self.korean.reset();
                self.literal.pop().is_some()
            }
            Reading::Korean => {
                self.literal.clear();
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
                    self.reading = Reading::Korean;
                }
                true
            }
            Reading::Mixed {
                boundary,
                order: SegmentOrder::EnglishFirst,
            } => {
                let consumed = self.mixed_korean.backspace();
                self.literal.truncate(boundary);
                if self.mixed_korean.byte_len() == 0 {
                    self.korean.reset();
                    self.reading = Reading::English;
                }
                consumed
            }
        };
        // Editing freezes the current interpretation. Replaying fewer raw keys would
        // erase syllable boundaries (갑사 + Backspace must become 갑ㅅ, not 값).
        self.edited = true;
        if !consumed {
            self.reset();
        } else if self.literal.is_empty() && self.korean.byte_len() == 0 {
            self.clear_composition();
        }
        self.update(consumed, String::new())
    }

    fn finish(&mut self) -> Update {
        self.context.clear();
        self.commit()
    }

    fn commit_word(&mut self) -> Update {
        let update = self.commit();
        if !update.commit.is_empty() {
            self.context.observe(update.language);
        }
        update
    }

    fn commit(&mut self) -> Update {
        let language = self.reading.language();
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
            language: self.reading.language(),
        }
    }

    fn select_reading(&mut self) {
        self.reading = match self.mode {
            Mode::Korean => Reading::Korean,
            Mode::English => Reading::English,
            Mode::Auto => self
                .decoder
                .decide(&self.literal, &self.korean.text(), &self.context),
        };
        if let Reading::Mixed { boundary, order } = self.reading {
            let korean_keys = match order {
                SegmentOrder::KoreanFirst => &self.literal[..boundary],
                SegmentOrder::EnglishFirst => &self.literal[boundary..],
            };
            self.mixed_korean.recompose(korean_keys);
        }
    }

    fn preedit(&self) -> String {
        match self.reading {
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
