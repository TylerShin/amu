use std::{collections::HashSet, sync::OnceLock};

const WORDS: &str = include_str!("english-seed.txt");
const ALPHABET_SIZE: usize = 27;
const WORD_EDGE: usize = 26;
const SMOOTHING: f32 = 0.5;
const INITIAL_CLUSTERS: &[&[u8]] = &[
    b"bl", b"br", b"ch", b"cl", b"cr", b"dr", b"fl", b"fr", b"gl", b"gr", b"ph", b"pl", b"pr",
    b"sc", b"sh", b"sk", b"sl", b"sm", b"sn", b"sp", b"st", b"sw", b"th", b"tr", b"tw", b"wh",
];
static MODEL: OnceLock<EnglishModel> = OnceLock::new();

pub(crate) struct EnglishEvidence {
    pub cost: f32,
    pub supports_split: bool,
    pub short_word_or_capitals: bool,
    pub capitals: bool,
}

pub(crate) fn initialize() {
    MODEL.get_or_init(EnglishModel::train);
}

/// Spelling and abbreviation shape are evidence even when no word hint matches.
pub(crate) fn assess(keys: &str) -> EnglishEvidence {
    let model = MODEL.get_or_init(EnglishModel::train);
    let lowercase = keys.to_ascii_lowercase();
    let length = keys.len() as f32;
    let word_hint = model.words.contains(lowercase.as_str());
    let capitals = keys.len() >= 2
        && keys.bytes().all(|key| key.is_ascii_uppercase())
        && keys.bytes().any(|key| !b"REQTWOP".contains(&key));
    let mut cost = model.spelling_cost(lowercase.as_bytes());
    if has_simple_word_shape(lowercase.as_bytes()) {
        cost = cost.min(length * 1.1);
    }

    // A short initialism need not resemble a pronounceable English word.
    // This depends only on shape, never on a list of registered abbreviations.
    if (3..=6).contains(&keys.len()) && !keys.bytes().all(|key| key == keys.as_bytes()[0]) {
        let vowels = lowercase
            .bytes()
            .filter(|key| b"aeiou".contains(key))
            .count();
        let per_letter = if vowels == 0 { 1.5 } else { 1.7 };
        cost = cost.min(length * per_letter);
    }
    if capitals {
        cost = cost.min(length * 0.7);
    }
    if word_hint {
        cost = cost.min(length * 0.7);
    } else if model.prefixes.contains(lowercase.as_str()) {
        cost = cost.min(length * 1.5);
    }
    EnglishEvidence {
        cost,
        supports_split: keys.len() >= 3 || capitals || (keys.len() >= 2 && word_hint),
        short_word_or_capitals: word_hint || capitals,
        capitals,
    }
}

fn has_simple_word_shape(keys: &[u8]) -> bool {
    if keys.len() < 4 {
        return false;
    }
    let mut vowels = 0;
    let mut consonant_run = 0;
    for (index, key) in keys.iter().enumerate() {
        if b"aeiouy".contains(key) {
            vowels += 1;
            consonant_run = 0;
        } else {
            consonant_run += 1;
            if consonant_run > 2 || (*key == b'q' && keys.get(index + 1) != Some(&b'u')) {
                return false;
            }
        }
    }
    let starts_with_cluster = !b"aeiouy".contains(&keys[0]) && !b"aeiouy".contains(&keys[1]);
    vowels > 0
        && vowels * 5 <= keys.len() * 3
        && (!starts_with_cluster || INITIAL_CLUSTERS.contains(&&keys[..2]))
}

struct EnglishModel {
    transition_costs: [[f32; ALPHABET_SIZE]; ALPHABET_SIZE],
    words: HashSet<&'static str>,
    prefixes: HashSet<&'static str>,
}

impl EnglishModel {
    fn train() -> Self {
        let words: HashSet<_> = WORDS.split_ascii_whitespace().collect();
        let mut prefixes = HashSet::new();
        let mut counts = [[SMOOTHING; ALPHABET_SIZE]; ALPHABET_SIZE];
        for word in &words {
            let mut previous = WORD_EDGE;
            for (index, key) in word.bytes().enumerate() {
                let current = usize::from(key - b'a');
                counts[previous][current] += 1.0;
                previous = current;
                prefixes.insert(&word[..=index]);
            }
            counts[previous][WORD_EDGE] += 1.0;
        }
        let mut transition_costs = counts;
        for row in &mut transition_costs {
            let total: f32 = row.iter().sum();
            for cost in row {
                *cost = -(*cost / total).ln() * 0.65;
            }
        }
        Self {
            transition_costs,
            words,
            prefixes,
        }
    }

    fn spelling_cost(&self, keys: &[u8]) -> f32 {
        let mut previous = WORD_EDGE;
        let mut cost = 0.0;
        for key in keys {
            let current = usize::from(key - b'a');
            cost += self.transition_costs[previous][current];
            previous = current;
        }
        cost + self.transition_costs[previous][WORD_EDGE] * 0.25
    }
}
