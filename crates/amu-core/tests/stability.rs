#[path = "support/keyboard.rs"]
mod keyboard;
#[path = "support/typing_trace.rs"]
mod typing_trace;

use amu_core::{HangulComposer, Input, Mode, Session};
use typing_trace::{Metrics, Trace, automatic, intended};

const CONTEXTS: &[&str] = &["", "gksrmf gksrmf gksrmf gksrmf", "hello hello hello hello"];

#[test]
fn supported_korean_paths_follow_the_composer_at_every_key() {
    for word in [
        "한글",
        "한국어",
        "안녕하세요",
        "어떻게",
        "어떨까",
        "벌써",
        "삼성",
        "돼지갈비",
        "싶네요",
        "입력기",
        "설정",
        "값",
        "갑사",
        "읽어",
        "닭과",
        "넓다",
        "괜찮아요",
        "썼어요",
    ] {
        let expected = intended(&format!("k:{}", keyboard::korean_keys(word)));
        for context in CONTEXTS {
            let actual = automatic(&expected.keys, context);
            // These supported paths allow a language choice after the first key.
            // Broader ambiguous cases remain in the intent evaluation cohort.
            assert_eq!(
                &actual.frames[1..],
                &expected.frames[1..],
                "{context}: {word}"
            );
            assert_eq!(actual.committed, word);
        }
    }
}

#[test]
fn active_compound_codas_and_repeated_initials_do_not_revert_to_latin() {
    let expected = intended("k:answkd");
    for context in CONTEXTS {
        let actual = automatic(&expected.keys, context);
        assert_eq!(&actual.frames[2..], &expected.frames[2..]);
    }

    for (keys, frames) in [
        ("fdzz", "f|ㄹㅇ|ㄹㅇㅋ|ㄹㅇㅋㅋ"),
        ("rtrt", "r|ㄱㅅ|ㄱㅅㄱ|ㄱㅅㄱㅅ"),
        ("zzz", "z|ㅋㅋ|ㅋㅋㅋ"),
    ] {
        for context in CONTEXTS {
            assert_eq!(
                automatic(keys, context).frames.join("|"),
                frames,
                "{context}: {keys}"
            );
        }
    }
}

#[test]
fn english_continuations_do_not_relabel_an_established_prefix() {
    for word in [
        "lock", "mock", "pick", "kick", "lick", "muck", "unlock", "deadlock", "fallback", "vexql",
    ] {
        for context in CONTEXTS {
            let expected = intended(&format!("e:{word}"));
            let actual = automatic(word, context);
            assert_eq!(actual.frames, expected.frames, "{context}: {word}");
            assert_eq!(actual.committed, word);
        }
    }
    for word in ["typescript", "florpdb"] {
        for context in CONTEXTS {
            let actual = automatic(word, context);
            let expected = intended(&format!("e:{word}"));
            assert_eq!(&actual.frames[2..], &expected.frames[2..]);
        }
    }
}

#[test]
fn pending_korean_onsets_do_not_hide_literal_recovery() {
    for word in [
        "bottles", "fields", "gpsa", "qandahar", "settled", "weekends", "worlds",
    ] {
        for context in CONTEXTS {
            let actual = automatic(word, context);
            assert_eq!(actual.committed, word, "{context}: {word}");
            assert_eq!(actual.frames.last(), Some(&actual.committed));
        }
    }
}

#[test]
fn open_english_spelling_does_not_override_a_completed_korean_word() {
    for context in CONTEXTS {
        let actual = automatic(&keyboard::korean_keys("닭무침이"), context);
        assert_eq!(actual.committed, "닭무침이");
    }
}

#[test]
fn a_korean_suffix_keeps_its_english_prefix_while_composing() {
    for (keys, frames) in [
        ("airh", "a|ai|air|ai고"),
        ("AIgksrmf", "A|AI|AIg|AI하|AI한|AI한ㄱ|AI한그|AI한글"),
        (
            "vexqlqnxj",
            "v|ve|vex|vexq|vexql|vexqlq|vexql부|vexql붙|vexql부터",
        ),
    ] {
        for context in CONTEXTS {
            let actual = automatic(keys, context);
            assert_eq!(actual.frames.join("|"), frames, "{context}: {keys}");
            assert_eq!(actual.frames.last(), Some(&actual.committed));
        }
    }
}

#[test]
fn fixed_korean_matches_the_composer_at_every_key_and_deletion_for_all_syllables() {
    let mut session = Session::new(Mode::Korean).unwrap();
    let mut composer = HangulComposer::new().unwrap();
    for scalar in 0xac00..=0xd7a3 {
        let syllable = char::from_u32(scalar).unwrap().to_string();
        let keys = keyboard::korean_keys(&syllable);
        session.reset();
        composer.reset();
        for key in keys.bytes() {
            let expected = composer.push(key);
            let actual = session.process(Input::Letter(key));
            assert_eq!(actual.preedit, expected.preedit, "{syllable}: append");
            assert!(actual.commit.is_empty());
        }
        for _ in keys.bytes() {
            let expected = composer.backspace();
            let actual = session.process(Input::Backspace);
            assert_eq!(actual.preedit, expected.preedit, "{syllable}: delete");
            assert_eq!(actual.consumed, expected.consumed);
        }
    }
}

#[test]
fn editing_and_mode_boundaries_do_not_reuse_display_evidence() {
    let mut session = Session::new(Mode::Auto).unwrap();
    for key in b"rkqtk" {
        session.process(Input::Letter(*key));
    }
    assert_eq!(session.process(Input::Backspace).preedit, "갑ㅅ");
    assert_eq!(session.process(Input::Letter(b'k')).preedit, "갑사");
    assert_eq!(session.process(Input::Finish).commit, "갑사");
    for key in b"lock" {
        session.process(Input::Letter(*key));
    }
    assert_eq!(session.change_mode(Mode::Korean).commit, "lock");
    for key in b"lock" {
        session.process(Input::Letter(*key));
    }
    assert_eq!(
        session.change_mode(Mode::Auto).commit,
        intended("k:lock").committed
    );
    for key in b"lock" {
        session.process(Input::Letter(*key));
    }
    assert_eq!(session.process(Input::WordBoundary).commit, "lock");
    session.reset();
    assert_eq!(session.process(Input::Letter(b'r')).preedit, "r");
}

#[test]
fn stability_metrics_count_reinterpretations_but_not_normal_composition() {
    let expected = intended("k:gksrmf");
    let mut correct = Metrics::default();
    correct.observe(&expected, &expected);
    assert_eq!(
        (
            correct.correct_frames,
            correct.relabeled_keys,
            correct.oscillating_keys
        ),
        (6, 0, 0)
    );

    let flicker = Trace {
        keys: "gksrmf".into(),
        committed: "한글".into(),
        frames: "g|하|한|gksr|한그|한글"
            .split('|')
            .map(str::to_owned)
            .collect(),
    };
    let mut metrics = Metrics::default();
    metrics.observe(&expected, &flicker);
    assert_eq!(
        (
            metrics.correct,
            metrics.correct_frames,
            metrics.relabeled_keys,
            metrics.oscillating_keys,
            metrics.stable_at
        ),
        (1, 4, 8, 3, 5)
    );

    let wrong = intended("e:gksrmf");
    let mut metrics = Metrics::default();
    metrics.observe(&expected, &wrong);
    assert_eq!(
        (
            metrics.correct,
            metrics.correct_frames,
            metrics.relabeled_keys
        ),
        (0, 0, 0)
    );
    assert_eq!(metrics.stable_at, 7);
}
