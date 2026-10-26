pub fn korean_keys(text: &str) -> String {
    const ONSETS: [&str; 19] = [
        "r", "R", "s", "e", "E", "f", "a", "q", "Q", "t", "T", "d", "w", "W", "c", "z", "x", "v",
        "g",
    ];
    const VOWELS: [&str; 21] = [
        "k", "o", "i", "O", "j", "p", "u", "P", "h", "hk", "ho", "hl", "y", "n", "nj", "np", "nl",
        "b", "m", "ml", "l",
    ];
    const CODAS: [&str; 28] = [
        "", "r", "R", "rt", "s", "sw", "sg", "e", "f", "fr", "fa", "fq", "ft", "fx", "fv", "fg",
        "a", "q", "qt", "t", "T", "d", "w", "c", "z", "x", "v", "g",
    ];
    let mut keys = String::new();
    for character in text.chars() {
        let syllable = character as usize - 0xac00;
        keys.push_str(ONSETS[syllable / 588]);
        keys.push_str(VOWELS[syllable % 588 / 28]);
        keys.push_str(CODAS[syllable % 28]);
    }
    keys
}
