use crate::language::{Reading, SegmentOrder};

const REINTERPRETED_KEY_COST: f32 = 0.75;
const MAX_REVISION_COST: f32 = 2.0;

/// A bounded cost for converting already displayed Latin keys on weak evidence.
/// Recovering literal text stays possible; supported whole Korean and Korean
/// prefixes are exempt at the call site. Explicit selection and deletion bypass automatic selection.
#[derive(Clone, Copy, Default)]
pub(crate) struct DisplayPolicy {
    previous: Option<Reading>,
    established_keys: usize,
}

impl DisplayPolicy {
    pub(crate) fn after(previous: Reading, previous_keys: usize) -> Self {
        Self {
            previous: Some(previous),
            established_keys: previous_keys,
        }
    }

    pub(crate) fn revision_cost(self, reading: Reading) -> f32 {
        // Restoring raw input and composing initials do not reinterpret an
        // established Latin span as a Korean syllable.
        if matches!(reading, Reading::English | Reading::Initials) {
            return 0.0;
        }
        let Some(previous) = self.previous else {
            return 0.0;
        };
        let changed = (0..self.established_keys)
            .filter(|key| is_english(previous, *key) && !is_english(reading, *key))
            .count();
        (changed as f32 * REINTERPRETED_KEY_COST).min(MAX_REVISION_COST)
    }

    pub(crate) fn continues_korean(self, boundary: usize) -> bool {
        matches!((self.previous, boundary), (Some(Reading::Korean), 0))
            || self.previous
                == Some(Reading::Mixed {
                    boundary,
                    order: SegmentOrder::EnglishFirst,
                })
    }

    pub(crate) fn continues_english(self) -> bool {
        self.previous == Some(Reading::English)
    }

    pub(crate) fn continues_initials(self) -> bool {
        self.previous == Some(Reading::Initials)
    }
}

fn is_english(reading: Reading, key: usize) -> bool {
    match reading {
        Reading::English => true,
        Reading::Korean | Reading::Initials => false,
        Reading::Mixed {
            boundary,
            order: SegmentOrder::EnglishFirst,
        } => key < boundary,
        Reading::Mixed {
            boundary,
            order: SegmentOrder::KoreanFirst,
        } => key >= boundary,
    }
}
