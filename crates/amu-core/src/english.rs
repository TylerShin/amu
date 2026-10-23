use std::{collections::HashSet, sync::OnceLock};

const WORDS: &str = include_str!("english-seed.txt");
const ALPHABET_SIZE: usize = 27;
const WORD_EDGE: usize = 26;
const SMOOTHING: f32 = 0.5;
const INITIAL_CLUSTERS: &[&[u8]] = &[
    b"bl", b"br", b"ch", b"cl", b"cr", b"dr", b"fl", b"fr", b"gl", b"gr", b"ph", b"pl", b"pr",
    b"sc", b"sh", b"sk", b"sl", b"sm", b"sn", b"sp", b"st", b"sw", b"th", b"tr", b"tw", b"wh",
];
const LONG_INITIAL_CLUSTERS: &[&[u8]] = &[b"scr", b"shr", b"spl", b"spr", b"str", b"thr"];
static MODEL: OnceLock<EnglishModel> = OnceLock::new();

pub(crate) struct EnglishEvidence {
    pub cost: f32,
    pub supports_split: bool,
    pub word_hint: bool,
    pub prefix_hint: bool,
    pub capitals: bool,
    pub connected_spelling: bool,
}

pub(crate) fn initialize() {
    MODEL.get_or_init(EnglishModel::train);
}

/// Spelling and abbreviation shape are evidence even when no word hint matches.
pub(crate) fn assess(keys: &str, use_capitals: bool) -> EnglishEvidence {
    let model = MODEL.get_or_init(EnglishModel::train);
    let lowercase = keys.to_ascii_lowercase();
    let length = keys.len() as f32;
    // A case-folded dictionary match must not erase meaningful Korean Shift.
    let word_hint = model.words.contains(lowercase.as_str()) && (use_capitals || keys == lowercase);
    let prefix_hint = model.prefixes.contains(lowercase.as_str());
    let capitals = use_capitals
        && keys.len() >= 2
        && keys.bytes().all(|key| key.is_ascii_uppercase())
        && keys.bytes().any(|key| !b"REQTWOP".contains(&key));
    let connected_spelling = model.has_connected_spelling(lowercase.as_bytes());
    let mut cost = model.spelling_cost(lowercase.as_bytes());
    if has_simple_word_shape(lowercase.as_bytes()) || (keys.len() >= 4 && connected_spelling) {
        cost = cost.min(length * 1.1);
    }

    // A short initialism need not resemble a pronounceable English word.
    // This depends only on shape, never on a list of registered abbreviations.
    if (2..=6).contains(&keys.len()) && !keys.bytes().all(|key| key == keys.as_bytes()[0]) {
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
    } else if prefix_hint {
        cost = cost.min(length * 1.5);
    }
    EnglishEvidence {
        cost,
        supports_split: keys.len() >= 3 || capitals || (keys.len() >= 2 && word_hint),
        word_hint,
        prefix_hint,
        capitals,
        connected_spelling,
    }
}

impl EnglishEvidence {
    pub(crate) fn continues_spelling(&self, keys: &str, boundary: usize) -> bool {
        let lowercase = keys.to_ascii_lowercase();
        // A short prefix can gain word evidence from the next unambiguous
        // English vowel. A consonant-only continuation still needs a stem.
        let adds_vowel = lowercase.as_bytes()[boundary..]
            .iter()
            .any(|key| b"aeiou".contains(key));
        (self.connected_spelling || adds_vowel)
            && MODEL
                .get_or_init(EnglishModel::train)
                .has_connected_spelling(lowercase.as_bytes())
    }
}

pub(crate) fn supports_transition(keys: &str, boundary: usize) -> bool {
    let left = keys.as_bytes()[boundary - 1].to_ascii_lowercase();
    let right = keys.as_bytes()[boundary].to_ascii_lowercase();
    MODEL.get_or_init(EnglishModel::train).transitions[usize::from(left - b'a')]
        [usize::from(right - b'a')]
}

fn has_simple_word_shape(keys: &[u8]) -> bool {
    if keys.len() < 4 {
        return false;
    }
    let vowels = vowel_count(keys);
    let mut consonant_run = 0;
    for (index, key) in keys.iter().enumerate() {
        if b"aeiouy".contains(key) {
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
    codas: HashSet<&'static [u8]>,
    transitions: [[bool; 26]; 26],
}

impl EnglishModel {
    fn train() -> Self {
        let words: HashSet<_> = WORDS.split_ascii_whitespace().collect();
        let mut prefixes = HashSet::new();
        let mut codas = HashSet::from([&b""[..]]);
        let mut counts = [[SMOOTHING; ALPHABET_SIZE]; ALPHABET_SIZE];
        let mut transitions = [[false; 26]; 26];
        for word in &words {
            let coda_start = word
                .bytes()
                .rposition(|key| b"aeiouy".contains(&key))
                .map_or(0, |index| index + 1);
            codas.insert(&word.as_bytes()[coda_start..]);
            let mut previous = WORD_EDGE;
            for (index, key) in word.bytes().enumerate() {
                let current = usize::from(key - b'a');
                counts[previous][current] += 1.0;
                if previous != WORD_EDGE {
                    transitions[previous][current] = true;
                }
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
            codas,
            transitions,
        }
    }

    fn has_connected_spelling(&self, keys: &[u8]) -> bool {
        // Two-letter initialisms do not establish the same continuity as a word.
        if keys.len() < 3 {
            return false;
        }
        let vowels = vowel_count(keys);
        if vowels == 0 || vowels * 5 > keys.len() * 3 {
            return false;
        }
        let mut index = 0;
        while index < keys.len() {
            if b"aeiouy".contains(&keys[index]) {
                index += 1;
                continue;
            }
            let start = index;
            while index < keys.len() && !b"aeiouy".contains(&keys[index]) {
                if keys[index] == b'q' && keys.get(index + 1) != Some(&b'u') {
                    return false;
                }
                index += 1;
            }
            let cluster = &keys[start..index];
            let supported = if start == 0 {
                is_onset(cluster)
            } else if index == keys.len() {
                self.codas.contains(cluster)
            } else {
                // Compound boundaries can join a coda to another onset. The
                // component words need not occur in the training vocabulary.
                (cluster.len().saturating_sub(3)..=cluster.len()).any(|split| {
                    self.codas.contains(&cluster[..split]) && is_onset(&cluster[split..])
                })
            };
            if !supported {
                return false;
            }
        }
        true
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

fn is_onset(cluster: &[u8]) -> bool {
    cluster.len() <= 1
        || INITIAL_CLUSTERS.contains(&cluster)
        || LONG_INITIAL_CLUSTERS.contains(&cluster)
}

fn vowel_count(keys: &[u8]) -> usize {
    // The u in the consonant spelling qu does not form another vowel nucleus.
    // Keep it as a separator in cluster parsing, but exclude it from density.
    keys.iter()
        .enumerate()
        .filter(|&(index, key)| {
            b"aeiouy".contains(key) && !(*key == b'u' && index > 0 && keys[index - 1] == b'q')
        })
        .count()
}
