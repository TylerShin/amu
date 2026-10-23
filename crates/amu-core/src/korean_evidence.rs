pub(crate) struct KoreanEvidence {
    pub complete: bool,
    pub ordinary: bool,
    pub repeated_jamo: bool,
    pub cost: f32,
}

impl KoreanEvidence {
    pub(crate) fn assess(text: &str, keys: usize) -> Self {
        let mut uncombined = 0;
        let mut compound_codas = 0;
        let mut unusual_rimes = 0;
        for character in text.chars() {
            if ('가'..='힣').contains(&character) {
                let coda = (character as u32 - 0xAC00) % 28;
                let vowel = (character as u32 - 0xAC00) % 588 / 28;
                if matches!(coda, 3 | 5 | 6 | 9..=15 | 18) {
                    compound_codas += 1;
                }
                // Diphthongs with a final aspirated stop are structurally valid
                // but weak evidence for an ordinary Korean word (e.g. 윁).
                if matches!(vowel, 9..=11 | 14..=16 | 19) && matches!(coda, 23..=26) {
                    unusual_rimes += 1;
                }
            } else {
                uncombined += 1;
            }
        }
        Self {
            complete: !text.is_empty() && uncombined == 0,
            ordinary: !text.is_empty() && uncombined == 0 && unusual_rimes == 0,
            repeated_jamo: uncombined >= 2 && has_repeated_jamo(text),
            cost: keys as f32
                + uncombined as f32 * 7.0
                + compound_codas as f32 * 1.5
                + unusual_rimes as f32 * 6.0,
        }
    }

    pub(crate) fn explains_capitals(&self, keys: &str, text: &str) -> bool {
        if !self.ordinary && !self.repeated_jamo {
            return false;
        }
        let mut in_shift_run = false;
        let mut has_capitals = false;
        for key in keys.bytes() {
            if key.is_ascii_lowercase() {
                in_shift_run = false;
                continue;
            }
            // Shift can remain held after a tense consonant or shifted vowel.
            // If every run starts with such a key, case alone is ambiguous.
            if !in_shift_run && !b"REQTWOP".contains(&key) {
                return false;
            }
            in_shift_run = true;
            has_capitals = true;
        }
        !has_capitals
            || self.repeated_jamo
            || crate::korean_statistics::contains_only_observed_syllables(text)
    }

    /// Composability alone is not language evidence. Corpus coverage or an explicit
    /// shifted Korean key can support an otherwise unfamiliar whole reading.
    pub(crate) fn supports_reading(&self, keys: &str, text: &str) -> bool {
        let explicit_shift = keys.bytes().any(|key| key.is_ascii_lowercase())
            && keys.bytes().any(|key| b"REQTWOP".contains(&key));
        self.complete
            && ((crate::korean_statistics::contains_observed_syllable(text)
                && crate::korean_statistics::supports_syllables(text))
                || explicit_shift)
    }

    /// A prefix may end mid-syllable. Completed syllables followed by jamo, or
    /// multiple complete syllables, are reasons not to relabel it as an acronym.
    pub(crate) fn supports_continuation(&self, keys: &str, text: &str) -> bool {
        let syllables = text.chars().filter(|c| ('가'..='힣').contains(c)).count();
        self.supports_reading(keys, text)
            || self.repeated_jamo
            || syllables > 1
            || (syllables == 1 && !self.complete)
    }
}

fn has_repeated_jamo(text: &str) -> bool {
    let mut characters = text.chars().peekable();
    let mut has_repetition = false;
    while let Some(character) = characters.next() {
        if ('가'..='힣').contains(&character) {
            continue;
        }
        if !('ㄱ'..='ㅣ').contains(&character) {
            return false;
        }
        let mut count = 1;
        while characters.peek() == Some(&character) {
            characters.next();
            count += 1;
        }
        if count < 2 {
            return false;
        }
        has_repetition = true;
    }
    has_repetition
}
