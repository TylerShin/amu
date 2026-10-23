use crate::{
    ComposerError,
    context::RecentContext,
    display_policy::DisplayPolicy,
    english,
    english_identifier::IdentifierProbe,
    korean_buffer::KoreanBuffer,
    korean_evidence::KoreanEvidence,
    korean_initials,
    selection::{Candidates, Decision},
};

const LANGUAGE_SWITCH_COST: f32 = 2.0;
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
    Initials,
    Mixed {
        boundary: usize,
        order: SegmentOrder,
    },
}

pub(crate) struct Decoder {
    probe: KoreanBuffer,
    identifiers: IdentifierProbe,
}

impl Decoder {
    pub(crate) fn new() -> Result<Self, ComposerError> {
        english::initialize();
        crate::korean_statistics::initialize();
        crate::initial_statistics::initialize();
        Ok(Self {
            probe: KoreanBuffer::new()?,
            identifiers: IdentifierProbe::new()?,
        })
    }

    pub(crate) fn decide(
        &mut self,
        keys: &str,
        korean: &KoreanBuffer,
        context: &RecentContext,
        policy: DisplayPolicy,
    ) -> Decision {
        if keys.is_empty() {
            return Decision::empty();
        }
        self.candidates(keys, korean, context, policy).select()
    }

    pub(crate) fn manual_readings(
        &mut self,
        keys: &str,
        korean: &KoreanBuffer,
        context: &RecentContext,
    ) -> Vec<Reading> {
        let korean_reading = if korean_initials::can_render(keys) {
            Reading::Initials
        } else {
            Reading::Korean
        };
        self.candidates(keys, korean, context, DisplayPolicy::default())
            .manual_readings(korean_reading)
    }

    pub(crate) fn reset(&mut self) {
        self.probe.reset();
        self.identifiers.reset();
    }

    fn candidates(
        &mut self,
        keys: &str,
        korean: &KoreanBuffer,
        context: &RecentContext,
        policy: DisplayPolicy,
    ) -> Candidates {
        let korean_text = korean.text();
        let mut korean_evidence = KoreanEvidence::assess(&korean_text, keys.len());
        let use_capitals = !korean_evidence.explains_capitals(keys, &korean_text);
        let mut english_evidence = self.identifiers.assess(keys, use_capitals);
        if policy.continues_korean(0) {
            korean_evidence.continue_composition(
                korean,
                english_evidence.complete_spelling
                    || english_evidence.word_hint
                    || english_evidence.identifier,
            );
        }
        let korean = korean_text.as_str();
        // An open English prefix can compete with unfinished Hangul. Once the
        // entire Hangul reading is complete, require whole-word English evidence.
        if policy.continues_english() && english_evidence.open_spelling && !korean_evidence.complete
        {
            english_evidence.cost = english_evidence.cost.min(keys.len() as f32 * 1.1);
        }
        let mut candidates = whole_candidates(
            keys,
            korean,
            &korean_evidence,
            &english_evidence,
            context,
            use_capitals,
            policy,
        );
        if let Some(cost) = korean_initials::candidate_cost(keys, &english_evidence) {
            candidates.consider(Reading::Initials, cost, 0.0);
        } else if policy.continues_initials()
            && korean_initials::can_render(keys)
            && let Some(cost) =
                korean_initials::candidate_cost(&keys[..keys.len() - 1], &english_evidence)
        {
            candidates.consider(Reading::Initials, cost, 0.0);
        }
        if keys.len() >= 4 {
            self.consider_korean_first(keys, context, &mut candidates, use_capitals, policy);
            let whole_korean_supported =
                crate::korean_statistics::contains_only_observed_syllables(korean);
            self.consider_english_first(
                keys,
                context,
                &mut candidates,
                use_capitals,
                whole_korean_supported,
                policy,
            );
        }
        candidates
    }

    fn consider_korean_first(
        &mut self,
        keys: &str,
        context: &RecentContext,
        candidates: &mut Candidates,
        use_capitals: bool,
        policy: DisplayPolicy,
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
                let reading = Reading::Mixed {
                    boundary,
                    order: SegmentOrder::KoreanFirst,
                };
                candidates.consider(
                    reading,
                    mixed_cost(
                        &prefix,
                        &korean_text,
                        &suffix,
                        context.cost(Language::Korean),
                        BoundaryEvidence::Shape,
                    ),
                    if prefix.supports_reading(&keys[..boundary], &korean_text) {
                        0.0
                    } else {
                        policy.revision_cost(reading)
                    },
                );
            }
        }
    }

    fn consider_english_first(
        &mut self,
        keys: &str,
        context: &RecentContext,
        candidates: &mut Candidates,
        use_capitals: bool,
        whole_korean_supported: bool,
        policy: DisplayPolicy,
    ) {
        let whole_english = english::assess(keys, use_capitals);
        let whole_english_supported = whole_english.word_hint;
        for boundary in 2..keys.len() {
            let prefix_keys = &keys[..boundary];
            let suffix_keys = &keys[boundary..];
            let prefix = self.identifiers.assess(prefix_keys, use_capitals);
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
            let mut suffix = KoreanEvidence::assess(&korean_text, suffix_keys.len());
            // An incomplete suffix needs evidence on both sides of its boundary;
            // raw prefix length alone must not keep a mistaken split alive.
            if policy.continues_korean(boundary)
                && (prefix.complete_spelling
                    || prefix.word_hint
                    || prefix.identifier
                    || prefix.capitals)
            {
                suffix.continue_composition(
                    &self.probe,
                    whole_english.complete_spelling || whole_english.word_hint,
                );
            }
            if !suffix.complete && !suffix.pending_onset {
                continue;
            }
            if !prefix.word_hint
                && !prefix.capitals
                && self.continues_english_spelling(keys, boundary, &korean_text, &prefix)
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
            let identifier_cost =
                self.identifiers
                    .boundary_cost(keys, boundary, &korean_text, use_capitals);
            let reading = Reading::Mixed {
                boundary,
                order: SegmentOrder::EnglishFirst,
            };
            candidates.consider(
                reading,
                mixed_cost(
                    &suffix,
                    &korean_text,
                    &prefix,
                    context.cost(Language::English),
                    evidence,
                ) + identifier_cost,
                policy.revision_cost(reading),
            );
        }
    }

    fn continues_english_spelling(
        &mut self,
        keys: &str,
        boundary: usize,
        korean: &str,
        prefix: &english::EnglishEvidence,
    ) -> bool {
        let Some(first_syllable) = korean.chars().next() else {
            return false;
        };
        self.probe.recompose(&keys[..boundary]);
        let prefix_text = self.probe.text();
        let prefix_korean = KoreanEvidence::assess(&prefix_text, boundary);
        // A stem that competes with complete Hangul needs a witnessed letter
        // junction before absorbing another composable syllable as English.
        if prefix_korean.complete && !english::supports_transition(keys, boundary) {
            return false;
        }
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
                return prefix.continues_spelling(
                    &keys[..boundary + index + 1],
                    boundary,
                    prefix_korean.complete,
                );
            }
        }
        false
    }

    fn prefers_english_prefix(&mut self, keys: &str, english: &english::EnglishEvidence) -> bool {
        self.probe.recompose(keys);
        let text = self.probe.text();
        let korean = KoreanEvidence::assess(&text, keys.len());
        let has_korean_support = if english.word_hint
            || english.capitals
            || english.identifier
            || (korean.complete
                && english.connected_spelling
                && !crate::korean_statistics::supports_syllables(&text))
        {
            korean.supports_reading(keys, &text) || korean.repeated_jamo
        } else {
            korean.supports_continuation(keys, &text)
        };
        !has_korean_support
            && english.cost < korean.cost + korean_coverage_cost(keys, &text, &korean)
    }
}

fn whole_candidates(
    keys: &str,
    korean: &str,
    korean_evidence: &KoreanEvidence,
    english_evidence: &english::EnglishEvidence,
    context: &RecentContext,
    use_capitals: bool,
    policy: DisplayPolicy,
) -> Candidates {
    let has_korean_support = korean_evidence.supports_reading(keys, korean);
    // A language prior resolves competing word evidence; it cannot manufacture it.
    let use_context =
        has_korean_support && english_evidence.word_hint && !english_evidence.capitals;
    let mut korean_cost =
        korean_evidence.cost + korean_coverage_cost(keys, korean, korean_evidence);
    if korean_evidence.repeated_jamo
        && !use_capitals
        && !korean_initials::has_expression_shape(keys)
    {
        korean_cost = keys.len() as f32 * 0.5;
    }
    let mut english_cost = english_evidence.cost;
    if use_context {
        korean_cost += context.cost(Language::Korean);
        english_cost += context.cost(Language::English);
    }
    Candidates::new(
        korean_cost,
        english_cost,
        if has_korean_support {
            0.0
        } else {
            policy.revision_cost(Reading::Korean)
        },
    )
}

impl Reading {
    pub(crate) fn language(self) -> Language {
        match self {
            Self::English => Language::English,
            Self::Korean | Self::Initials => Language::Korean,
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
    if korean.complete && !korean.active_syllable_supported && !korean.supports_reading(keys, text)
    {
        UNSUPPORTED_KOREAN_COST
    } else {
        0.0
    }
}
