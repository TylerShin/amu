use amu_core::{HangulComposer, Input, Language, Mode, Session};

#[test]
fn every_modern_syllable_composes_and_deletes() {
    let onsets = [
        "r", "R", "s", "e", "E", "f", "a", "q", "Q", "t", "T", "d", "w", "W", "c", "z", "x", "v",
        "g",
    ];
    let vowels = [
        "k", "o", "i", "O", "j", "p", "u", "P", "h", "hk", "ho", "hl", "y", "n", "nj", "np", "nl",
        "b", "m", "ml", "l",
    ];
    let codas = [
        "", "r", "R", "rt", "s", "sw", "sg", "e", "f", "fr", "fa", "fq", "ft", "fx", "fv", "fg",
        "a", "q", "qt", "t", "T", "d", "w", "c", "z", "x", "v", "g",
    ];
    let mut composer = HangulComposer::new().unwrap();
    for (onset, start) in onsets.iter().enumerate() {
        for (vowel, middle) in vowels.iter().enumerate() {
            for (coda, end) in codas.iter().enumerate() {
                composer.reset();
                let keys = format!("{start}{middle}{end}");
                let mut shown = String::new();
                for ascii in keys.bytes() {
                    let update = composer.push(ascii);
                    assert!(update.consumed);
                    assert!(update.settled.is_empty(), "unexpected early commit: {keys}");
                    shown = update.preedit;
                }
                let expected =
                    char::from_u32(0xAC00 + ((onset * 21 + vowel) * 28 + coda) as u32).unwrap();
                assert_eq!(shown, expected.to_string(), "{keys}");
                for _ in keys.bytes() {
                    let update = composer.backspace();
                    assert!(update.consumed, "early empty composition: {keys}");
                    shown = update.preedit;
                }
                assert!(shown.is_empty(), "{keys}");
                assert!(!composer.backspace().consumed);
            }
        }
    }
}

#[test]
fn backspace_preserves_settled_syllable_boundary() {
    let mut session = Session::new(Mode::Korean).unwrap();
    assert_eq!(type_keys(&mut session, "rkqtk"), "갑사");
    assert_eq!(session.process(Input::Backspace).preedit, "갑ㅅ");
    assert_eq!(session.process(Input::Backspace).preedit, "갑");
    assert_eq!(session.process(Input::Backspace).preedit, "");
    assert!(!session.process(Input::Backspace).consumed);
}

#[test]
fn korean_edit_does_not_reinterpret_remaining_raw_keys() {
    let mut session = Session::new(Mode::Auto).unwrap();
    assert_eq!(type_keys(&mut session, "rkqtk"), "갑사");
    assert_eq!(session.process(Input::Backspace).preedit, "갑ㅅ");
    assert_eq!(type_keys(&mut session, "k"), "갑사");
}

#[test]
fn auto_mode_handles_basic_mixed_words() {
    let mut session = Session::new(Mode::Auto).unwrap();
    for (keys, expected) in [
        ("hello", "hello"),
        ("dkssud", "안녕"),
        ("rust", "rust"),
        ("gksrmf", "한글"),
        ("go", "go"),
    ] {
        assert_eq!(type_keys(&mut session, keys), expected);
        let update = session.process(Input::Finish);
        assert_eq!(update.commit, expected);
        assert!(!update.consumed);
        assert!(update.preedit.is_empty());
        assert!(session.process(Input::Finish).commit.is_empty());
    }
}

#[test]
fn auto_mode_preserves_mixed_readings_after_korean() {
    let mut session = Session::new(Mode::Auto).unwrap();
    for (keys, expected) in [
        ("tkatjdsdi", "삼성sdi"),
        ("tkatjdSDI", "삼성SDI"),
        ("gksrmfapi", "한글api"),
        ("tjdsmddmlgpu", "성능의gpu"),
        ("sdi", "sdi"),
    ] {
        assert_eq!(type_keys(&mut session, keys), expected, "{keys}");
        assert_eq!(session.process(Input::Finish).commit, expected);
    }
    let mut sentence = type_keys(&mut session, "tkatjdsdi");
    assert_eq!(session.process(Input::Finish).commit, sentence);
    sentence.push(' ');
    sentence.push_str(&type_keys(&mut session, "tkrhtlvdj"));
    assert_eq!(sentence, "삼성sdi 사고싶어");
}

#[test]
fn mixed_suffix_deletion_preserves_both_language_boundaries() {
    let mut session = Session::new(Mode::Auto).unwrap();
    assert_eq!(type_keys(&mut session, "tkatjdsdi"), "삼성sdi");
    assert_eq!(session.process(Input::Backspace).preedit, "삼성sd");
    assert_eq!(type_keys(&mut session, "i"), "삼성sdi");
    for expected in ["삼성sd", "삼성s", "삼성", "삼서"] {
        assert_eq!(session.process(Input::Backspace).preedit, expected);
    }
    session.reset();
    assert_eq!(type_keys(&mut session, "rkqtkapi"), "갑사api");
    for expected in ["갑사ap", "갑사a", "갑사", "갑ㅅ", "갑", ""] {
        assert_eq!(session.process(Input::Backspace).preedit, expected);
    }
    assert!(!session.process(Input::Backspace).consumed);
    assert_eq!(type_keys(&mut session, "hello"), "hello");
}

#[test]
fn ambiguous_complete_korean_and_fixed_modes_are_not_segmented() {
    let mut session = Session::new(Mode::Auto).unwrap();
    for (keys, expected) in [("qnsdi", "분야"), ("dusdk", "연아"), ("dlsdk", "인아")] {
        assert_eq!(type_keys(&mut session, keys), expected);
        session.process(Input::Finish);
    }
    session.change_mode(Mode::Korean);
    assert_eq!(type_keys(&mut session, "tkatjdsdi"), "삼성ㄴ야");
    session.change_mode(Mode::English);
    assert_eq!(type_keys(&mut session, "tkatjdsdi"), "tkatjdsdi");
}

#[test]
fn mixed_readings_survive_mode_changes_and_the_token_size_limit() {
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "tkatjdsdi");
    let update = session.change_mode(Mode::Korean);
    assert_eq!(update.language, Language::Mixed);
    assert_eq!(update.commit, "삼성sdi");
    assert_eq!(type_keys(&mut session, "rk"), "가");
    session.change_mode(Mode::Auto);

    // 64 keys: the last complete reading must be committed before the next key.
    let prefix = format!("{}rks", "rk".repeat(29));
    let keys = format!("{prefix}api");
    let expected = format!("{}간api", "가".repeat(29));
    assert_eq!(type_keys(&mut session, &keys), expected);
    let update = session.process(Input::Letter(b'h'));
    assert_eq!(update.commit, expected);
    assert_eq!(type_keys(&mut session, "ello"), "hello");
    session.reset();
    assert!(session.process(Input::Finish).commit.is_empty());
}

#[test]
fn changing_mode_commits_once_and_starts_a_new_composition() {
    let mut session = Session::new(Mode::Korean).unwrap();
    type_keys(&mut session, "rhk");
    assert_eq!(session.change_mode(Mode::English).commit, "과");
    assert_eq!(type_keys(&mut session, "hello"), "hello");
    assert_eq!(session.process(Input::Backspace).preedit, "hell");
    session.reset();
    assert!(session.process(Input::Finish).commit.is_empty());
}

#[test]
fn long_tokens_are_committed_in_bounded_chunks_without_loss() {
    let mut session = Session::new(Mode::English).unwrap();
    let original = "a".repeat(4096);
    let mut committed = String::new();
    for ascii in original.bytes() {
        let update = session.process(Input::Letter(ascii));
        committed.push_str(&update.commit);
        assert!(update.preedit.len() <= 64);
    }
    committed.push_str(&session.process(Input::Finish).commit);
    assert_eq!(committed, original);
}

#[test]
fn sessions_do_not_share_composition() {
    let mut first = Session::new(Mode::Korean).unwrap();
    let mut second = Session::new(Mode::Korean).unwrap();
    assert_eq!(type_keys(&mut first, "rk"), "가");
    assert_eq!(type_keys(&mut second, "sk"), "나");
    assert_eq!(type_keys(&mut first, "s"), "간");
    assert_eq!(second.process(Input::Finish).commit, "나");
}

#[test]
fn deleting_an_english_token_clears_the_alternate_korean_state() {
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "hello");
    for _ in 0..5 {
        session.process(Input::Backspace);
    }
    assert_eq!(type_keys(&mut session, "rk"), "가");
}

fn type_keys(session: &mut Session, keys: &str) -> String {
    let mut preedit = String::new();
    for ascii in keys.bytes() {
        preedit = session.process(Input::Letter(ascii)).preedit;
    }
    preedit
}
