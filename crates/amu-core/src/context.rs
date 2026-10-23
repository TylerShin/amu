use crate::Language;

const CONTEXT_WORDS: usize = 4;

/// A bounded language prior. No committed text is retained.
#[derive(Default)]
pub(crate) struct RecentContext {
    votes: [i8; CONTEXT_WORDS],
    next: usize,
}

impl RecentContext {
    pub(crate) fn observe(&mut self, language: Option<Language>) {
        self.votes[self.next] = match language {
            Some(Language::English) => 1,
            Some(Language::Korean) => -1,
            Some(Language::Mixed) | None => 0,
        };
        self.next = (self.next + 1) % CONTEXT_WORDS;
    }

    pub(crate) fn cost(&self, language: Language) -> f32 {
        let bias = f32::from(self.votes.iter().sum::<i8>().clamp(-3, 3)) * 0.6;
        match language {
            Language::English => -bias,
            Language::Korean => bias,
            Language::Mixed => 0.0,
        }
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}
