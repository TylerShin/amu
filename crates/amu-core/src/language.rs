use crate::{
    ComposerError, context::RecentContext, english, korean_buffer::KoreanBuffer,
    korean_evidence::KoreanEvidence,
};

const LANGUAGE_SWITCH_COST: f32 = 2.0;
const SPLIT_CONFIDENCE_MARGIN: f32 = 0.75;
const CAPITAL_BOUNDARY_BONUS: f32 = 3.0;
const INDEPENDENT_BOUNDARY_BONUS: f32 = 3.0;
// Missing corpus coverage is weak evidence: it can resolve a minimal open
// syllable, but must not outweigh an additional composed consonant by itself.
const UNSUPPORTED_KOREAN_COST: f32 = 1.5;

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
        let mut best = unsplit_candidate(keys, korean, &korean_evidence, context, use_capitals);
        if keys.len() < 4 {
            return best.reading;
        }
        let unsplit_cost = best.cost;
        self.consider_korean_first(keys, context, &mut best, unsplit_cost, use_capitals);
        let whole_korean_supported =
            crate::korean_statistics::contains_only_observed_syllables(korean);
        self.consider_english_first(
            keys,
            context,
            &mut best,
            unsplit_cost,
            use_capitals,
            whole_korean_supported,
        );
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
        let whole_english = english::assess(keys, use_capitals);
        self.probe.reset();
        for boundary in 1..keys.len() {
            self.probe.append(keys.as_bytes()[boundary - 1]);
            let korean_text = self.probe.text();
            let prefix = KoreanEvidence::assess(&korean_text, boundary);
            let suffix = english::assess(&keys[boundary..], use_capitals);
            if whole_english.connected_spelling && !suffix.word_hint && !suffix.capitals {
                continue;
            }
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
                            BoundaryEvidence::Shape,
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
        whole_korean_supported: bool,
    ) {
        let whole_english_supported = english::assess(keys, use_capitals).word_hint;
        for boundary in 2..keys.len() {
            let prefix_keys = &keys[..boundary];
            let suffix_keys = &keys[boundary..];
            let prefix = english::assess(prefix_keys, use_capitals);
            // A familiar complete Korean reading must not be split just because
            // its first keys also spell a short English word.
            if whole_korean_supported && !prefix.capitals {
                continue;
            }
            let independent_prefix =
                !whole_english_supported && self.prefers_english_prefix(prefix_keys, &prefix);
            if !prefix.supports_split && !independent_prefix {
                continue;
            }
            self.probe.recompose(suffix_keys);
            let korean_text = self.probe.text();
            let suffix = KoreanEvidence::assess(&korean_text, suffix_keys.len());
            if !suffix.complete {
                continue;
            }
            if prefix.connected_spelling
                && !prefix.word_hint
                && !prefix.capitals
                && self.continues_english_spelling(keys, boundary, &korean_text)
            {
                continue;
            }
            let evidence = if independent_prefix
                && independently_reads_korean(suffix_keys, &korean_text, &suffix)
            {
                BoundaryEvidence::IndependentReadings
            } else {
                BoundaryEvidence::Shape
            };
            if !prefix.supports_split && matches!(evidence, BoundaryEvidence::Shape) {
                continue;
            }
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
                        evidence,
                    ),
                },
                unsplit_cost,
            );
        }
    }

    fn continues_english_spelling(&mut self, keys: &str, boundary: usize, korean: &str) -> bool {
        let Some(first_syllable) = korean.chars().next() else {
            return false;
        };
        self.probe.reset();
        // Ask libhangul which prefix produced the first syllable. Once a
        // second character appears, a one-syllable prefix cannot match again.
        for (index, key) in keys[boundary..].bytes().enumerate() {
            self.probe.append(key);
            let composed = self.probe.text();
            let mut characters = composed.chars();
            let first = characters.next();
            if characters.next().is_some() {
                return false;
            }
            if first == Some(first_syllable) {
                return english::has_connected_spelling(&keys[..boundary + index + 1]);
            }
        }
        false
    }

    fn prefers_english_prefix(&mut self, keys: &str, english: &english::EnglishEvidence) -> bool {
        self.probe.recompose(keys);
        let text = self.probe.text();
        let korean = KoreanEvidence::assess(&text, keys.len());
        let has_korean_support = if english.word_hint || english.capitals {
            korean.supports_reading(keys, &text) || korean.repeated_jamo
        } else {
            korean.supports_continuation(keys, &text)
        };
        !has_korean_support
            && english.cost < korean.cost + korean_coverage_cost(keys, &text, &korean)
    }
}

fn unsplit_candidate(
    keys: &str,
    korean: &str,
    korean_evidence: &KoreanEvidence,
    context: &RecentContext,
    use_capitals: bool,
) -> Candidate {
    let english_evidence = english::assess(keys, use_capitals);
    let has_korean_support = korean_evidence.supports_reading(keys, korean);
    // Context resolves competing word evidence. It cannot manufacture Korean
    // evidence or erase capitals that Korean Shift usage does not explain.
    let use_context =
        has_korean_support && english_evidence.word_hint && !english_evidence.capitals;
    let mut best = Candidate {
        reading: Reading::Korean,
        cost: korean_evidence.cost
            + korean_coverage_cost(keys, korean, korean_evidence)
            + if use_context {
                context.cost(Language::Korean)
            } else {
                0.0
            },
    };
    // Always compare both readings; structural validity must not veto Latin.
    best.consider(Candidate {
        reading: Reading::English,
        cost: english_evidence.cost
            + if use_context {
                context.cost(Language::English)
            } else {
                0.0
            },
    });
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
    evidence: BoundaryEvidence,
) -> f32 {
    if matches!(evidence, BoundaryEvidence::IndependentReadings) {
        // Two independently supported readings provide boundary evidence even
        // without a registered word or an uppercase transition.
        return korean.cost + english.cost - INDEPENDENT_BOUNDARY_BONUS;
    }
    let boundary_cost = if english.capitals {
        -CAPITAL_BOUNDARY_BONUS
    } else {
        crate::korean_statistics::boundary_cost(text)
    };
    let prior = if english.capitals {
        0.0
    } else {
        context_cost * 0.5
    };
    korean.cost + english.cost + LANGUAGE_SWITCH_COST + boundary_cost + prior
}

enum BoundaryEvidence {
    Shape,
    IndependentReadings,
}

fn independently_reads_korean(keys: &str, text: &str, korean: &KoreanEvidence) -> bool {
    let use_capitals = !korean.explains_capitals(keys, text);
    korean.ordinary
        && crate::korean_statistics::contains_only_observed_syllables(text)
        && korean.cost < english::assess(keys, use_capitals).cost
}

fn korean_coverage_cost(keys: &str, text: &str, korean: &KoreanEvidence) -> f32 {
    if korean.complete && !korean.supports_reading(keys, text) {
        UNSUPPORTED_KOREAN_COST
    } else {
        0.0
    }
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
