use crate::{ComposerError, HangulComposer, korean_initials};

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

    /// Keep explicitly selected initials separate; only the last onset is
    /// still available to combine with the newly entered vowel.
    pub(crate) fn continue_initials(&mut self, keys: &str) {
        let onset = keys.len() - 2;
        self.reset();
        self.settled = korean_initials::render(&keys[..onset]);
        self.append(keys.as_bytes()[onset]);
        self.append(keys.as_bytes()[onset + 1]);
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

    pub(crate) fn pending_onset_prefix(&self) -> Option<&str> {
        let pending = !self.settled.is_empty()
            && self.settled.chars().all(|c| ('가'..='힣').contains(&c))
            && self.syllable.chars().count() == 1
            && self.syllable.chars().all(|c| ('ㄱ'..='ㅎ').contains(&c));
        pending.then_some(self.settled.as_str())
    }

    pub(crate) fn active_syllable(&self) -> &str {
        &self.syllable
    }

    pub(crate) fn settled_text(&self) -> &str {
        &self.settled
    }
}
