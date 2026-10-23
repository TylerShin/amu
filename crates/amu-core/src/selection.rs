use crate::language::Reading;

const MIN_CONFIDENCE_MARGIN: f32 = 0.75;

/// Score gaps are heuristic evidence, not calibrated probabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionStatus {
    Idle,
    Supported,
    Ambiguous,
    Explicit,
    Edited,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Decision {
    pub reading: Reading,
    pub alternative: Option<Reading>,
    pub status: DecisionStatus,
    pub margin: Option<f32>,
}

pub(crate) struct Candidates {
    best: Candidate,
    runner_up: Candidate,
    whole: Candidate,
}

impl Candidates {
    pub(crate) fn new(korean_cost: f32, english_cost: f32) -> Self {
        let korean = Candidate {
            reading: Reading::Korean,
            cost: korean_cost,
        };
        let english = Candidate {
            reading: Reading::English,
            cost: english_cost,
        };
        let (best, runner_up) = if english_cost < korean_cost {
            (english, korean)
        } else {
            (korean, english)
        };
        Self {
            best,
            runner_up,
            whole: best,
        }
    }

    pub(crate) fn consider(&mut self, reading: Reading, cost: f32) {
        let candidate = Candidate { reading, cost };
        if cost < self.best.cost {
            self.runner_up = self.best;
            self.best = candidate;
        } else if cost < self.runner_up.cost {
            self.runner_up = candidate;
        }
    }

    pub(crate) fn select(self) -> Decision {
        let margin = self.runner_up.cost - self.best.cost;
        let status = if margin >= MIN_CONFIDENCE_MARGIN {
            DecisionStatus::Supported
        } else {
            DecisionStatus::Ambiguous
        };
        // A transformation must beat the whole reading by a margin. Two close
        // mixed boundaries can still both be much better than the whole token;
        // retain the best boundary but do not learn from that uncertain choice.
        let reading = if self.best.reading != self.whole.reading
            && self.whole.cost - self.best.cost < MIN_CONFIDENCE_MARGIN
        {
            self.whole.reading
        } else {
            self.best.reading
        };
        let alternative = if reading == self.best.reading {
            self.runner_up.reading
        } else {
            self.best.reading
        };
        Decision {
            reading,
            alternative: Some(alternative),
            status,
            margin: Some(margin),
        }
    }
}

impl Decision {
    pub(crate) fn empty() -> Self {
        Self {
            status: DecisionStatus::Idle,
            ..Self::fixed(Reading::English)
        }
    }

    pub(crate) fn fixed(reading: Reading) -> Self {
        Self {
            reading,
            alternative: None,
            status: DecisionStatus::Explicit,
            margin: None,
        }
    }
}

#[derive(Clone, Copy)]
struct Candidate {
    reading: Reading,
    cost: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::SegmentOrder;

    #[test]
    fn a_small_initials_advantage_preserves_literal_and_the_alternative() {
        let mut candidates = Candidates::new(16.0, 3.0);
        candidates.consider(Reading::Initials, 2.5);
        let selected = candidates.select();
        assert_eq!(selected.reading, Reading::English);
        assert_eq!(selected.alternative, Some(Reading::Initials));
        assert_eq!(selected.status, DecisionStatus::Ambiguous);
    }

    #[test]
    fn ambiguity_does_not_discard_an_existing_whole_korean_composition() {
        let selected = Candidates::new(2.0, 2.4).select();
        assert_eq!(selected.reading, Reading::Korean);
        assert_eq!(selected.status, DecisionStatus::Ambiguous);
        assert_eq!(selected.alternative, Some(Reading::English));
    }

    #[test]
    fn close_boundaries_do_not_erase_a_well_supported_mixed_reading() {
        let first = Reading::Mixed {
            boundary: 4,
            order: SegmentOrder::KoreanFirst,
        };
        let second = Reading::Mixed {
            boundary: 5,
            order: SegmentOrder::KoreanFirst,
        };
        for order in [[(first, 1.0), (second, 1.2)], [(second, 1.2), (first, 1.0)]] {
            let mut candidates = Candidates::new(12.0, 10.0);
            for (reading, cost) in order {
                candidates.consider(reading, cost);
            }
            let selected = candidates.select();
            assert_eq!(selected.reading, first);
            assert_eq!(selected.alternative, Some(second));
            assert_eq!(selected.status, DecisionStatus::Ambiguous);
        }
    }
}
