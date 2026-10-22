pub(crate) struct KoreanEvidence {
    pub complete: bool,
    pub ordinary: bool,
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
            cost: keys as f32
                + uncombined as f32 * 7.0
                + compound_codas as f32 * 1.5
                + unusual_rimes as f32 * 6.0,
        }
    }
}
