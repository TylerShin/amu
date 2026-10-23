use amu_core::{Input, Mode, Session};

pub struct Trace {
    pub keys: String,
    pub frames: Vec<String>,
    pub committed: String,
}

/// The intended language of each segment supplies the oracle, independently of
/// the automatic decoder. `e`, `k`, and `i` mean English, Korean, and initials.
pub fn intended(segments: &str) -> Trace {
    let mut keys = String::new();
    let mut frames = Vec::new();
    let mut committed = String::new();
    for segment in segments.split(' ') {
        let (language, letters) = segment.split_once(':').expect("language:keys");
        let mode = match language {
            "e" => Mode::English,
            "k" | "i" => Mode::Korean,
            _ => panic!("unknown intended language"),
        };
        let mut session = Session::new(mode).unwrap();
        for key in letters.bytes() {
            keys.push(key as char);
            let update = session.process(Input::Letter(key));
            frames.push(format!("{committed}{}", update.preedit));
            if language == "i" {
                committed.push_str(&session.process(Input::Finish).commit);
            }
        }
        committed.push_str(&session.process(Input::Finish).commit);
    }
    Trace {
        keys,
        frames,
        committed,
    }
}

pub fn automatic(keys: &str, context: &str) -> Trace {
    let mut session = Session::new(Mode::Auto).unwrap();
    for word in context.split_ascii_whitespace() {
        for key in word.bytes() {
            session.process(Input::Letter(key));
        }
        session.process(Input::WordBoundary);
    }
    let frames = keys
        .bytes()
        .map(|key| {
            let update = session.process(Input::Letter(key));
            assert!(update.commit.is_empty(), "trace fixture must fit one token");
            update.preedit
        })
        .collect();
    Trace {
        keys: keys.to_owned(),
        frames,
        committed: session.process(Input::WordBoundary).commit,
    }
}

#[derive(Default)]
pub struct Metrics {
    pub tokens: usize,
    pub correct: usize,
    pub frames: usize,
    pub correct_frames: usize,
    pub relabeled_keys: usize,
    pub oscillating_keys: usize,
    pub stable_at: usize,
}

impl Metrics {
    pub fn observe(&mut self, expected: &Trace, actual: &Trace) {
        assert_eq!(expected.frames.len(), actual.frames.len());
        self.tokens += 1;
        self.correct += usize::from(actual.committed == expected.committed);
        self.frames += actual.frames.len();
        self.correct_frames += actual
            .frames
            .iter()
            .zip(&expected.frames)
            .filter(|(actual, expected)| actual == expected)
            .count();
        // Last incorrect frame + 1: the key after which every remaining frame
        // matches the intended composition. n+1 means it never settled correctly.
        self.stable_at += actual
            .frames
            .iter()
            .zip(&expected.frames)
            .rposition(|(actual, expected)| actual != expected)
            .map_or(1, |index| index + 2);

        let mut previous = Vec::new();
        let mut changes = vec![0; actual.keys.len()];
        for (index, frame) in actual.frames.iter().enumerate() {
            let labels = key_languages(&actual.keys[..=index], frame);
            for (position, (before, after)) in previous.iter().zip(&labels).enumerate() {
                if before != after {
                    self.relabeled_keys += 1;
                    changes[position] += 1;
                }
            }
            previous = labels;
        }
        self.oscillating_keys += changes.iter().filter(|count| **count >= 2).count();
    }
}

/// Language attribution is observable without decoder internals: literal ASCII
/// occupies the same raw key at the start/end of a frame; the rest is Hangul.
/// Normal Hangul composition and coda movement therefore count as zero flips.
fn key_languages(keys: &str, frame: &str) -> Vec<bool> {
    let prefix = frame.bytes().take_while(u8::is_ascii_alphabetic).count();
    let suffix = frame
        .bytes()
        .rev()
        .take_while(u8::is_ascii_alphabetic)
        .count();
    assert!(prefix <= keys.len() && suffix <= keys.len());
    assert!(keys.starts_with(&frame[..prefix]));
    assert!(keys.ends_with(&frame[frame.len() - suffix..]));
    (0..keys.len())
        .map(|index| index < prefix || index >= keys.len() - suffix)
        .collect()
}
