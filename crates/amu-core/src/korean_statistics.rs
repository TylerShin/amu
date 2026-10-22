use std::{collections::HashMap, sync::OnceLock};

const DATA: &str = include_str!("../../../models/korean/syllables.tsv");
const SYLLABLES: usize = 11172;
const UNKNOWN_COST: f32 = 12.0;
const EXPECTED_TRANSITION_COST: f32 = 4.5;
const STATISTICS_WEIGHT: f32 = 0.75;
static MODEL: OnceLock<SyllableModel> = OnceLock::new();

pub(crate) fn initialize() {
    MODEL.get_or_init(SyllableModel::load);
}

/// Rates local syllable context at a proposed language boundary.
pub(crate) fn boundary_cost(text: &str) -> f32 {
    let model = MODEL.get_or_init(SyllableModel::load);
    let mut previous = None;
    let mut first_cost = 0.0;
    let mut transitions = 0;
    let mut transition_cost = 0.0;
    for character in text.chars() {
        if !('가'..='힣').contains(&character) {
            previous = None;
            continue;
        }
        let current = character as usize - 0xac00;
        if let Some(previous) = previous {
            transition_cost += model
                .bigrams
                .get(&(previous * SYLLABLES + current))
                .copied()
                .unwrap_or((model.unigrams[current] + 2.0).min(UNKNOWN_COST));
            transitions += 1;
        } else {
            first_cost = model.unigrams[current] * 0.1;
        }
        previous = Some(current);
    }
    if transitions == 0 {
        return first_cost;
    }
    // Normalize by length: raw log-cost otherwise favors cutting a Korean word
    // short simply because fewer syllables are left in its Korean segment.
    (first_cost
        + (transition_cost / transitions as f32 - EXPECTED_TRANSITION_COST) * STATISTICS_WEIGHT)
        .clamp(-2.0, 4.0)
}

/// Corpus support distinguishes plausible Shift overlap from a rare syllable
/// that an uppercase abbreviation happens to compose.
pub(crate) fn contains_only_observed_syllables(text: &str) -> bool {
    let model = MODEL.get_or_init(SyllableModel::load);
    text.chars().all(|character| {
        ('가'..='힣').contains(&character)
            && model.unigrams[character as usize - 0xac00] < UNKNOWN_COST
    })
}

pub(crate) fn contains_observed_syllable(text: &str) -> bool {
    let model = MODEL.get_or_init(SyllableModel::load);
    text.chars().any(|character| {
        ('가'..='힣').contains(&character)
            && model.unigrams[character as usize - 0xac00] < UNKNOWN_COST
    })
}

struct SyllableModel {
    unigrams: [f32; SYLLABLES],
    bigrams: HashMap<usize, f32>,
}

impl SyllableModel {
    fn load() -> Self {
        let mut model = Self {
            unigrams: [UNKNOWN_COST; SYLLABLES],
            bigrams: HashMap::new(),
        };
        for line in DATA.lines().filter(|line| !line.starts_with('#')) {
            let mut fields = line.split('\t');
            let kind = fields.next().expect("model row kind");
            let key: usize = fields
                .next()
                .expect("model key")
                .parse()
                .expect("numeric model key");
            let cost = fields
                .next()
                .expect("model cost")
                .parse()
                .expect("numeric model cost");
            match kind {
                "U" => model.unigrams[key] = cost,
                "B" => {
                    model.bigrams.insert(key, cost);
                }
                _ => panic!("unknown model row kind"),
            }
        }
        model
    }
}
