#[path = "support/keyboard.rs"]
mod keyboard;
#[path = "support/typing_trace.rs"]
mod typing_trace;

use amu_core::{Input, Mode, Session};

const CONTEXTS: &[&str] = &["", "gksrmf gksrmf gksrmf gksrmf", "hello hello hello hello"];

#[test]
fn complete_english_words_compete_with_individually_familiar_syllables() {
    for word in ["goal", "goals", "coal", "coals", "soap"] {
        assert!(
            !include_str!("../src/english-seed.txt")
                .split_ascii_whitespace()
                .any(|seed| seed == word)
        );
        for context in CONTEXTS {
            let actual = typing_trace::automatic(word, context);
            let intended = typing_trace::intended(&format!("e:{word}"));
            let mut metrics = typing_trace::Metrics::default();
            metrics.observe(&intended, &actual);
            assert_eq!(metrics.correct, 1, "{context}: {word}");
            assert_eq!(actual.frames.last().unwrap(), word, "{context}: {word}");
            assert_eq!(actual.committed, word);
        }
    }
}

#[test]
fn korean_connections_endings_shift_and_reduplication_are_preserved() {
    for word in [
        "미래",
        "소개해",
        "개냐",
        "재임",
        "걔가",
        "해째",
        "태그",
        "한글",
        "촘촘히",
        "돼지갈비",
        "야생동물의",
        "미약한",
        "핸드",
        "새해",
        "야생",
    ] {
        for context in CONTEXTS {
            assert_eq!(
                typing_trace::automatic(&keyboard::korean_keys(word), context).committed,
                word
            );
        }
    }
}

#[test]
fn literal_recovery_survives_deletion_and_retains_the_korean_alternative() {
    let mut session = Session::new(Mode::Auto).unwrap();
    for key in b"goal" {
        session.process(Input::Letter(*key));
    }
    assert_eq!(session.process(Input::Backspace).preedit, "goa");
    assert_eq!(session.process(Input::Letter(b'l')).preedit, "goal");
    assert_eq!(session.process(Input::ToggleLanguage).preedit, "해미");
    assert_eq!(session.process(Input::ToggleLanguage).preedit, "goal");
    assert_eq!(session.process(Input::WordBoundary).commit, "goal");
}
