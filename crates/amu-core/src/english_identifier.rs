use crate::{
    ComposerError, english, korean_buffer::KoreanBuffer, korean_evidence::KoreanEvidence,
    korean_statistics,
};

pub(crate) struct IdentifierProbe {
    composition: KoreanBuffer,
}

impl IdentifierProbe {
    pub(crate) fn new() -> Result<Self, ComposerError> {
        Ok(Self {
            composition: KoreanBuffer::new()?,
        })
    }

    pub(crate) fn reset(&mut self) {
        self.composition.reset();
    }

    /// Compare a stem plus initialism with its Korean suffix interpretation.
    /// This scratch composer never changes the session's real deletion state.
    pub(crate) fn assess(&mut self, keys: &str, use_capitals: bool) -> english::EnglishEvidence {
        let mut evidence = english::assess(keys, use_capitals);
        let cost = english::identifier_cost(keys, |suffix| {
            self.composition.recompose(suffix);
            let text = self.composition.text();
            let korean = KoreanEvidence::assess(&text, suffix.len());
            let mut characters = text.chars();
            let first = characters.next();
            korean.complete
                && characters.next().is_none()
                && first.is_some_and(korean_statistics::lacks_attachment_evidence)
        });
        if let Some(cost) = cost.filter(|cost| *cost < evidence.cost) {
            evidence.cost = cost;
            evidence.identifier = true;
        }
        evidence
    }

    /// Moving the boundary inside a supported identifier needs extra evidence,
    /// including when more Korean syllables have already followed the first one.
    pub(crate) fn boundary_cost(
        &mut self,
        keys: &str,
        boundary: usize,
        korean: &str,
        use_capitals: bool,
    ) -> f32 {
        if boundary < 3 {
            return 0.0;
        }
        let first = korean.chars().next().expect("complete Korean suffix");
        self.composition.reset();
        for (index, key) in keys[boundary..].bytes().enumerate() {
            self.composition.append(key);
            let text = self.composition.text();
            if text.chars().count() > 1 {
                break;
            }
            if text.starts_with(first) {
                let prefix = self.assess(&keys[..boundary + index + 1], use_capitals);
                return if prefix.identifier {
                    korean_statistics::attachment_cost(first)
                } else {
                    0.0
                };
            }
        }
        0.0
    }
}
