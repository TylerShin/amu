use amu_core::{Input, Mode, Session};

const CASES: &str = include_str!("fixtures/decoder.tsv");
const WORD_HINTS: &str = include_str!("../src/english-seed.txt");

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
