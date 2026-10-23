use std::sync::OnceLock;

const DATA: &str = include_str!("../../../models/initials/pairs.tsv");
const PAIRS: usize = 26 * 26;
static MODEL: OnceLock<[f32; PAIRS]> = OnceLock::new();

pub(crate) fn initialize() {
    MODEL.get_or_init(load);
}

/// Positive log odds support Korean onsets; negative values support Latin.
/// This is corpus-derived pair evidence, not a vocabulary of abbreviations.
pub(crate) fn log_odds(pair: &[u8]) -> f32 {
    let index = usize::from(pair[0] - b'a') * 26 + usize::from(pair[1] - b'a');
    MODEL.get_or_init(load)[index]
}

fn load() -> [f32; PAIRS] {
    let mut odds = [0.0; PAIRS];
    for line in DATA.lines().filter(|line| !line.starts_with('#')) {
        let mut fields = line.split('\t');
        let pair: usize = fields
            .next()
            .expect("pair key")
            .parse()
            .expect("numeric pair key");
        let korean: f32 = fields
            .next()
            .expect("Korean cost")
            .parse()
            .expect("numeric Korean cost");
        let english: f32 = fields
            .next()
            .expect("English cost")
            .parse()
            .expect("numeric English cost");
        odds[pair] = english - korean;
    }
    odds
}
