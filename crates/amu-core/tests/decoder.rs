use amu_core::{Input, Mode, Session};

#[path = "support/mixed_boundaries.rs"]
mod mixed_boundaries;

const CASES: &str = include_str!("fixtures/decoder.tsv");
const WORD_HINTS: &str = include_str!("../src/english-seed.txt");
const CONTEXTS: &[&[&str]] = &[
    &[],
    &["gksrmf"],
    &["gksrmf", "gksrmf", "gksrmf", "gksrmf"],
    &["hello"],
    &["hello", "hello", "hello", "hello"],
];

#[test]
fn unregistered_english_prefixes_compose_korean_suffixes_across_contexts() {
    for prefix in mixed_boundaries::UNREGISTERED_PREFIXES {
        assert!(
            !WORD_HINTS
                .split_ascii_whitespace()
                .any(|word| word.eq_ignore_ascii_case(prefix))
        );
        for (suffix, korean) in mixed_boundaries::KOREAN_SUFFIXES {
            for context in CONTEXTS {
                let keys = format!("{prefix}{suffix}");
                let expected = format!("{prefix}{korean}");
                let mut session = session_after(context);
                assert_eq!(
                    type_keys(&mut session, &keys),
                    expected,
                    "{context:?}: {keys}"
                );
                assert_eq!(session.process(Input::WordBoundary).commit, expected);
                assert!(session.process(Input::Finish).commit.is_empty());
            }
        }
    }
}

#[test]
fn short_mixed_readings_preserve_real_syllable_deletion_and_retyping() {
    for prefix in ["ai", "vi", "qi", "xy", "io"] {
        let mut session = Session::new(Mode::Auto).unwrap();
        assert_eq!(
            type_keys(&mut session, &format!("{prefix}rkqtk")),
            format!("{prefix}갑사")
        );
        assert_eq!(
            session.process(Input::Backspace).preedit,
            format!("{prefix}갑ㅅ")
        );
        assert_eq!(type_keys(&mut session, "k"), format!("{prefix}갑사"));
        for remaining in ["갑ㅅ", "갑", ""] {
            assert_eq!(
                session.process(Input::Backspace).preedit,
                format!("{prefix}{remaining}")
            );
        }
        assert_eq!(session.process(Input::Backspace).preedit, &prefix[..1]);
        assert!(session.process(Input::Backspace).preedit.is_empty());
        assert_eq!(type_keys(&mut session, "gksrmf"), "한글");
    }
}

#[test]
fn complete_korean_words_are_not_split_at_english_looking_prefixes() {
    for &(keys, expected) in mixed_boundaries::KOREAN_WORDS {
        for context in CONTEXTS {
            let mut session = session_after(context);
            assert_eq!(
                type_keys(&mut session, keys),
                expected,
                "{context:?}: {keys}"
            );
        }
    }
}

#[test]
fn english_words_are_not_split_at_composable_suffixes() {
    for &word in mixed_boundaries::ENGLISH_WORDS {
        for context in CONTEXTS {
            let mut session = session_after(context);
            assert_eq!(type_keys(&mut session, word), word, "{context:?}: {word}");
            assert_eq!(session.process(Input::WordBoundary).commit, word);
        }
    }
}

#[test]
fn short_readings_survive_case_context_and_editing_boundaries() {
    for line in CASES
        .lines()
        .filter(|line| line.starts_with("short-english\t") || line.starts_with("korean-single\t"))
    {
        let fields: Vec<_> = line.split('\t').collect();
        let keys = fields[1];
        let expected = fields[2];
        for context in CONTEXTS {
            let mut session = session_after(context);
            assert_eq!(
                type_keys(&mut session, keys),
                expected,
                "{context:?}: {keys}"
            );
            assert_eq!(session.process(Input::WordBoundary).commit, expected);
            assert!(session.process(Input::Finish).commit.is_empty());
        }
    }
    for keys in ["ai", "AI", "Ai", "aI"] {
        let mut session = session_after(&["gksrmf", "gksrmf"]);
        assert_eq!(type_keys(&mut session, keys), keys);
        assert_eq!(session.process(Input::Backspace).preedit, &keys[..1]);
        assert_eq!(type_keys(&mut session, &keys[1..]), keys);
        assert_eq!(session.process(Input::Finish).commit, keys);
    }
}

#[test]
fn context_cannot_override_two_letter_capitals_without_a_korean_shift_onset() {
    // Exhaust the 494 pairs with no possible tense-consonant/shifted-vowel
    // onset. Pairs such as RK remain ambiguous with intentional Korean Shift.
    for first in b'A'..=b'Z' {
        if b"REQTWOP".contains(&first) {
            continue;
        }
        for second in b'A'..=b'Z' {
            let keys = String::from_utf8(vec![first, second]).unwrap();
            for context in CONTEXTS {
                let mut session = session_after(context);
                assert_eq!(type_keys(&mut session, &keys), keys, "{context:?}");
                assert_eq!(session.process(Input::WordBoundary).commit, keys);
            }
        }
    }
}

#[test]
fn capital_segments_and_korean_shift_overlap_survive_opposing_context() {
    for context in CONTEXTS {
        for (keys, expected) in [
            ("AIgksrmf", "AI한글"),
            ("gksrmfAI", "한글AI"),
            ("AIrk", "AI가"),
            ("djEjfRK", "어떨까"),
            ("qjfTJ", "벌써"),
            ("dkRK", "아까"),
        ] {
            let mut session = session_after(context);
            assert_eq!(
                type_keys(&mut session, keys),
                expected,
                "{context:?}: {keys}"
            );
            assert_eq!(session.process(Input::WordBoundary).commit, expected);
        }
    }
}

#[test]
fn explicit_modes_preserve_both_intentions_for_the_same_ambiguous_keys() {
    for (mode, expected) in [(Mode::Korean, "먀"), (Mode::English, "ai")] {
        let mut session = Session::new(mode).unwrap();
        assert_eq!(type_keys(&mut session, "ai"), expected);
        assert_eq!(session.process(Input::Finish).commit, expected);
    }
}

#[test]
fn unseen_abbreviations_and_normal_text_are_decoded_together() {
    let mut failures = Vec::new();
    for line in CASES.lines().filter(|line| !line.starts_with('#')) {
        let fields: Vec<_> = line.split('\t').collect();
        let [category, keys, expected, unknown] = fields.as_slice() else {
            panic!("invalid fixture")
        };
        if *unknown != "-" {
            assert!(
                !WORD_HINTS
                    .split_ascii_whitespace()
                    .any(|word| word.eq_ignore_ascii_case(unknown)),
                "held-out term leaked into word hints: {unknown}"
            );
        }
        let mut session = Session::new(Mode::Auto).unwrap();
        type_keys(&mut session, keys);
        let actual = session.process(Input::Finish).commit;
        if actual != *expected {
            failures.push(format!(
                "{category}: {keys} → {actual} (expected {expected})"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn recent_words_resolve_an_ambiguous_key_sequence_and_reset_at_focus_boundaries() {
    let mut korean = Session::new(Mode::Auto).unwrap();
    type_keys(&mut korean, "gksrmf");
    assert_eq!(korean.process(Input::WordBoundary).commit, "한글");
    assert_eq!(type_keys(&mut korean, "go"), "해");
    korean.process(Input::Finish);
    assert_eq!(type_keys(&mut korean, "go"), "go");

    let mut english = Session::new(Mode::Auto).unwrap();
    for word in ["we", "can"] {
        type_keys(&mut english, word);
        english.process(Input::WordBoundary);
    }
    assert_eq!(type_keys(&mut english, "go"), "go");
    english.process(Input::WordBoundary);
    assert_eq!(type_keys(&mut english, "rk"), "가");
    english.process(Input::WordBoundary);
    assert_eq!(type_keys(&mut english, "eogks"), "대한");
    korean.reset();
    assert_eq!(type_keys(&mut korean, "go"), "go");
}

#[test]
fn shift_held_after_tense_consonants_does_not_split_a_korean_sentence() {
    let mut session = Session::new(Mode::Auto).unwrap();
    let mut sentence = String::new();
    for (keys, expected) in [
        ("wj", "저"),
        ("qjfTJ", "벌써"),
        ("dkRK", "아까"),
        ("rmrj", "그거"),
        ("ek", "다"),
        ("aksemfdjTdjdy", "만들었어요"),
    ] {
        assert_eq!(type_keys(&mut session, keys), expected, "{keys}");
        sentence.push_str(&session.process(Input::WordBoundary).commit);
        sentence.push(' ');
    }
    assert_eq!(sentence.trim_end(), "저 벌써 아까 그거 다 만들었어요");
}

#[test]
fn korean_words_survive_shift_overlapping_the_following_keys() {
    for line in CASES.lines().filter(|line| line.starts_with("korean\t")) {
        let fields: Vec<_> = line.split('\t').collect();
        let keys = fields[1];
        let expected = fields[2];
        for (start, key) in keys.bytes().enumerate() {
            if !b"REQTWOP".contains(&key) {
                continue;
            }
            let mut held = keys.as_bytes().to_vec();
            for end in start + 1..held.len() {
                // Stop where holding Shift would change the intended Hangul.
                if b"reqtwop".contains(&held[end]) {
                    break;
                }
                held[end].make_ascii_uppercase();
                let variant = std::str::from_utf8(&held).unwrap();
                let mut session = Session::new(Mode::Auto).unwrap();
                assert_eq!(type_keys(&mut session, variant), expected, "{variant}");
            }
        }
    }
}

#[test]
fn shifted_final_syllable_stays_korean_before_a_word_boundary() {
    for context in ["", "gksrmf", "hello"] {
        for keys in ["djEjfRk", "djEjfRK", "djEJfRK", "djEJFRK"] {
            let mut session = Session::new(Mode::Auto).unwrap();
            type_keys(&mut session, context);
            session.process(Input::WordBoundary);
            assert_eq!(type_keys(&mut session, keys), "어떨까", "{context}: {keys}");
            assert_eq!(session.process(Input::WordBoundary).commit, "어떨까");
            assert_eq!(type_keys(&mut session, "tlvspdy"), "싶네요");
        }
    }
}

#[test]
fn repeated_jamo_supports_editing_boundaries_and_explicit_english() {
    let mut session = Session::new(Mode::Auto).unwrap();
    for (keys, expected) in [("ggg", "ㅎㅎㅎ"), ("zzz", "ㅋㅋㅋ")] {
        assert_eq!(type_keys(&mut session, keys), expected);
        for remaining in (0..3).rev() {
            let update = session.process(Input::Backspace);
            assert!(update.consumed);
            assert_eq!(
                update.preedit,
                expected.chars().take(remaining).collect::<String>()
            );
        }
        assert!(!session.process(Input::Backspace).consumed);
    }
    assert_eq!(type_keys(&mut session, "zzz"), "ㅋㅋㅋ");
    assert_eq!(session.process(Input::WordBoundary).commit, "ㅋㅋㅋ");
    assert_eq!(type_keys(&mut session, "hello"), "hello");
    session.process(Input::Finish);
    assert_eq!(type_keys(&mut session, "ZZZ"), "ZZZ");
    session.change_mode(Mode::English);
    assert_eq!(type_keys(&mut session, "ggg"), "ggg");
    session.process(Input::WordBoundary);
    assert_eq!(type_keys(&mut session, "zzz"), "zzz");
}

#[test]
fn deleting_committed_text_discards_stale_context() {
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "gksrmf");
    session.process(Input::WordBoundary);
    assert!(!session.process(Input::Backspace).consumed);
    assert_eq!(type_keys(&mut session, "go"), "go");
}

#[test]
fn mixed_edit_sequences_remain_bounded_and_reset_cleanly() {
    let mut session = Session::new(Mode::Auto).unwrap();
    let mut seed: u64 = 0x631a;
    for _ in 0..10_000 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let input = match seed % 20 {
            0..=2 => Input::Backspace,
            3 => Input::WordBoundary,
            4 => Input::Finish,
            _ => Input::Letter(b'a' + ((seed >> 32) % 26) as u8),
        };
        let update = session.process(input);
        assert!(update.commit.len() < 1024 && update.preedit.len() < 1024);
    }
    session.reset();
    assert!(session.process(Input::Finish).commit.is_empty());
    assert_eq!(type_keys(&mut session, "tkatjdqzx"), "삼성qzx");
}

#[test]
fn editing_a_korean_suffix_keeps_the_english_prefix_and_syllable_boundaries() {
    let mut session = Session::new(Mode::Auto).unwrap();
    assert_eq!(type_keys(&mut session, "githubrkqtk"), "github갑사");
    assert_eq!(session.process(Input::Backspace).preedit, "github갑ㅅ");
    assert_eq!(type_keys(&mut session, "k"), "github갑사");
    for expected in ["github갑ㅅ", "github갑", "github", "githu"] {
        assert_eq!(session.process(Input::Backspace).preedit, expected);
    }
    assert_eq!(session.process(Input::Finish).commit, "githu");
}

fn type_keys(session: &mut Session, keys: &str) -> String {
    let mut shown = String::new();
    for key in keys.bytes() {
        shown = session.process(Input::Letter(key)).preedit;
    }
    shown
}

fn session_after(words: &[&str]) -> Session {
    let mut session = Session::new(Mode::Auto).unwrap();
    for keys in words {
        type_keys(&mut session, keys);
        session.process(Input::WordBoundary);
    }
    session
}
