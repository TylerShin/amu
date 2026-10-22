use crate::{ComposerError, HangulComposer};

/// Keeps completed syllables separate from libhangul's still-editable syllable.
pub(crate) struct KoreanBuffer {
    composer: HangulComposer,
    settled: String,
    syllable: String,
}

impl KoreanBuffer {
    pub(crate) fn new() -> Result<Self, ComposerError> {
        Ok(Self {
            composer: HangulComposer::new()?,
            settled: String::new(),
            syllable: String::new(),
        })
    }

    pub(crate) fn append(&mut self, ascii: u8) {
        let composition = self.composer.push(ascii);
        self.settled.push_str(&composition.settled);
        self.syllable = composition.preedit;
    }

    pub(crate) fn backspace(&mut self) -> bool {
        let composition = self.composer.backspace();
        self.syllable = composition.preedit;
        composition.consumed || self.settled.pop().is_some()
    }

    /// Used to evaluate a new candidate, never to replay a deletion.
    pub(crate) fn recompose(&mut self, keys: &str) {
        self.reset();
        for ascii in keys.bytes() {
            self.append(ascii);
        }
    }

    pub(crate) fn reset(&mut self) {
        self.composer.reset();
        self.settled.clear();
        self.syllable.clear();
    }

    pub(crate) fn text(&self) -> String {
        format!("{}{}", self.settled, self.syllable)
    }

    pub(crate) fn byte_len(&self) -> usize {
        self.settled.len() + self.syllable.len()
    }
}
