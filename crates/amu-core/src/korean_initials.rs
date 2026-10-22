/// A distinct onset pair, its repetition, or a pair followed by repeated reactions.
/// Longer arbitrary consonant strings remain ambiguous with Latin initialisms.
pub(crate) fn has_expression_shape(keys: &str) -> bool {
    if keys.len() < 2 || !keys.bytes().all(|key| initial(key).is_some()) {
        return false;
    }
    let bytes = keys.as_bytes();
    // Identical onsets belong to the existing repeated-jamo policy.
    if bytes[0] == bytes[1] {
        return false;
    }
    let pair = &bytes[..2];
    if bytes.chunks(2).all(|chunk| chunk == pair) {
        return true;
    }
    let mut tail = &bytes[2..];
    while let Some(first) = tail.first() {
        let count = tail.iter().take_while(|key| *key == first).count();
        if count < 2 {
            return false;
        }
        tail = &tail[count..];
    }
    true
}

/// Unlike syllable composition, an initial expression has one character per key.
pub(crate) fn render(keys: &str) -> String {
    keys.bytes()
        .map(|key| initial(key).expect("initial reading contains only consonant keys"))
        .collect()
}

// The unshifted two-set keyboard, not a vocabulary of expressions. Uppercase
// remains explicit Latin evidence; shifted Korean still uses the composer.
fn initial(key: u8) -> Option<char> {
    Some(match key {
        b'r' => 'ㄱ',
        b's' => 'ㄴ',
        b'e' => 'ㄷ',
        b'f' => 'ㄹ',
        b'a' => 'ㅁ',
        b'q' => 'ㅂ',
        b't' => 'ㅅ',
        b'd' => 'ㅇ',
        b'w' => 'ㅈ',
        b'c' => 'ㅊ',
        b'z' => 'ㅋ',
        b'x' => 'ㅌ',
        b'v' => 'ㅍ',
        b'g' => 'ㅎ',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HangulComposer;

    #[test]
    fn initial_keys_match_the_real_keyboard() {
        let mut composer = HangulComposer::new().unwrap();
        for key in b'a'..=b'z' {
            composer.reset();
            let composed = composer.push(key).preedit;
            let consonant = composed
                .chars()
                .next()
                .filter(|c| ('ㄱ'..='ㅎ').contains(c));
            assert_eq!(initial(key), consonant);
        }
    }
}
