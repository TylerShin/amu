use crate::{
    ComposerError, context::RecentContext, english, korean_buffer::KoreanBuffer,
    korean_evidence::KoreanEvidence,
};

const LANGUAGE_SWITCH_COST: f32 = 2.0;
const SPLIT_CONFIDENCE_MARGIN: f32 = 0.75;
const CAPITAL_BOUNDARY_BONUS: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Korean,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SegmentOrder {
    KoreanFirst,
    EnglishFirst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reading {
    English,
    Korean,
    Mixed {
        boundary: usize,
        order: SegmentOrder,
    },
}

pub(crate) struct Decoder {
    probe: KoreanBuffer,
}

impl Decoder {
    pub(crate) fn new() -> Result<Self, ComposerError> {
        english::initialize();
        crate::korean_statistics::initialize();
        Ok(Self {
            probe: KoreanBuffer::new()?,
        })
    }

    pub(crate) fn decide(&mut self, keys: &str, korean: &str, context: &RecentContext) -> Reading {
        if keys.is_empty() {
            return Reading::English;
        }
        let korean_evidence = KoreanEvidence::assess(korean, keys.len());
        let use_capitals = !korean_evidence.explains_capitals(keys, korean);
        if korean_evidence.repeated_jamo && !use_capitals {
            return Reading::Korean;
        }
        let mut best = unsplit_candidate(keys, &korean_evidence, context, use_capitals);
        if keys.len() < 4 {
            return best.reading;
        }
        let unsplit_cost = best.cost;
        self.consider_korean_first(keys, context, &mut best, unsplit_cost, use_capitals);
        self.consider_english_first(keys, context, &mut best, unsplit_cost, use_capitals);
        best.reading
    }

    pub(crate) fn reset(&mut self) {
        self.probe.reset();
    }

    fn consider_korean_first(
        &mut self,
        keys: &str,
        context: &RecentContext,
        best: &mut Candidate,
        unsplit_cost: f32,
        use_capitals: bool,
    ) {
        self.probe.reset();
        for boundary in 1..keys.len() {
            self.probe.append(keys.as_bytes()[boundary - 1]);
            let korean_text = self.probe.text();
            let prefix = KoreanEvidence::assess(&korean_text, boundary);
            let suffix = english::assess(&keys[boundary..], use_capitals);
            if prefix.complete && suffix.supports_split {
                best.consider_split(
                    Candidate {
                        reading: Reading::Mixed {
                            boundary,
                            order: SegmentOrder::KoreanFirst,
                        },
                        cost: mixed_cost(
                            &prefix,
                            &korean_text,
                            &suffix,
                            context.cost(Language::Korean),
                        ),
                    },
                    unsplit_cost,
                );
            }
        }
    }

    fn consider_english_first(
        &mut self,
        keys: &str,
        context: &RecentContext,
        best: &mut Candidate,
        unsplit_cost: f32,
        use_capitals: bool,
    ) {
        for boundary in 2..keys.len() {
            let prefix = english::assess(&keys[..boundary], use_capitals);
            if !prefix.supports_split {
                continue;
            }
            self.probe.recompose(&keys[boundary..]);
            let korean_text = self.probe.text();
            let suffix = KoreanEvidence::assess(&korean_text, keys.len() - boundary);
            if suffix.complete {
                best.consider_split(
                    Candidate {
                        reading: Reading::Mixed {
                            boundary,
                            order: SegmentOrder::EnglishFirst,
                        },
                        cost: mixed_cost(
                            &suffix,
                            &korean_text,
                            &prefix,
                            context.cost(Language::English),
                        ),
                    },
                    unsplit_cost,
                );
            }
        }
    }
}

fn unsplit_candidate(
    keys: &str,
    korean_evidence: &KoreanEvidence,
    context: &RecentContext,
    use_capitals: bool,
) -> Candidate {
    let english_evidence = english::assess(keys, use_capitals);
    let use_context = !korean_evidence.complete || english_evidence.short_word_or_capitals;
    let mut best = Candidate {
        reading: Reading::Korean,
        cost: korean_evidence.cost
            + if use_context {
                context.cost(Language::Korean)
            } else {
                0.0
            },
    };
    // An unfamiliar abbreviation's shape alone cannot disprove complete,
    // structurally ordinary Korean. Preserve that reading when ambiguous.
    if !korean_evidence.ordinary || english_evidence.short_word_or_capitals {
        best.consider(Candidate {
            reading: Reading::English,
            cost: english_evidence.cost
                + if use_context {
                    context.cost(Language::English)
                } else {
                    0.0
                },
        });
    }
    best
}

impl Reading {
    pub(crate) fn language(self) -> Language {
        match self {
            Self::English => Language::English,
            Self::Korean => Language::Korean,
            Self::Mixed { .. } => Language::Mixed,
        }
    }
}

fn mixed_cost(
    korean: &KoreanEvidence,
    text: &str,
    english: &english::EnglishEvidence,
    context_cost: f32,
) -> f32 {
    let boundary_cost = if english.capitals {
        -CAPITAL_BOUNDARY_BONUS
    } else {
        crate::korean_statistics::boundary_cost(text)
    };
    korean.cost + english.cost + LANGUAGE_SWITCH_COST + boundary_cost + context_cost * 0.5
}

struct Candidate {
    reading: Reading,
    cost: f32,
}

impl Candidate {
    fn consider(&mut self, candidate: Self) {
        if candidate.cost < self.cost {
            *self = candidate;
        }
    }

    fn consider_split(&mut self, candidate: Self, unsplit_cost: f32) {
        if candidate.cost + SPLIT_CONFIDENCE_MARGIN < unsplit_cost {
            self.consider(candidate);
        }
    }
}
