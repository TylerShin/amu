use amu_core::{DecisionStatus, HangulComposer, Input, Mode, Session};

#[test]
fn an_unfamiliar_name_can_be_corrected_without_a_dictionary_entry() {
    let mut session = Session::new(Mode::Auto).unwrap();
    assert_eq!(type_keys(&mut session, "dia"), "얌");
    let corrected = session.process(Input::CycleReading);
    assert_eq!(corrected.preedit, "dia");
    assert!(corrected.consumed && corrected.commit.is_empty());
    assert_eq!(session.interpretation().status, DecisionStatus::Explicit);
    assert_eq!(session.process(Input::WordBoundary).commit, "dia");
    assert_eq!(type_keys(&mut session, "browser"), "browser");
}

#[test]
fn manual_choice_survives_continuation_and_returns_to_auto_at_a_boundary() {
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "di");
    choose(&mut session, "di");
    assert_eq!(type_keys(&mut session, "a"), "dia");
    assert_eq!(session.process(Input::Backspace).preedit, "di");
    assert_eq!(type_keys(&mut session, "a"), "dia");
    assert_eq!(session.process(Input::WordBoundary).commit, "dia");
    assert_eq!(type_keys(&mut session, "dkssud"), "안녕");

    session.reset();
    type_keys(&mut session, "ai");
    choose(&mut session, "먀");
    assert_eq!(type_keys(&mut session, "d"), "먕");
    assert_eq!(session.process(Input::WordBoundary).commit, "먕");
    assert_eq!(type_keys(&mut session, "query"), "query");
}

#[test]
fn the_same_keys_allow_both_latin_and_korean_intentions() {
    for (keys, korean) in [("fd", "ㄹㅇ"), ("rt", "ㄱㅅ"), ("vs", "ㅍㄴ"), ("ai", "먀")] {
        let mut session = Session::new(Mode::Auto).unwrap();
        type_keys(&mut session, keys);
        choose(&mut session, keys);
        choose(&mut session, korean);
        assert_eq!(session.process(Input::WordBoundary).commit, korean);
        assert_eq!(type_keys(&mut session, "djEjfRk"), "어떨까");
    }
}

#[test]
fn unregistered_key_sequences_have_both_whole_readings() {
    for keys in ["qev", "zup", "vax", "diaq", "xoq", "kiv", "jux", "nuv"] {
        let mut session = Session::new(Mode::Auto).unwrap();
        let korean = compose(keys);
        type_keys(&mut session, keys);
        choose(&mut session, keys);
        choose(&mut session, &korean);
        choose(&mut session, keys);
        assert_eq!(session.process(Input::Finish).commit, keys);
    }
}

#[test]
fn mixed_candidates_remain_available_and_keep_their_selected_boundary() {
    for (keys, expected, tail, continued) in [
        ("githubdp", "github에", "tj", "github에서"),
        ("queryfmf", "query를", "d", "query를ㅇ"),
        ("airh", "ai고", "d", "ai공"),
        ("tkatjdsdi", "삼성sdi", "a", "삼성sdia"),
    ] {
        let mut session = Session::new(Mode::Auto).unwrap();
        type_keys(&mut session, keys);
        choose(&mut session, keys);
        choose(&mut session, expected);
        assert_eq!(type_keys(&mut session, tail), continued);
        assert_eq!(session.process(Input::Finish).commit, continued);
    }
}

#[test]
fn korean_deletion_is_never_replayed_as_a_shorter_key_sequence() {
    for (keys, original, deleted) in [
        ("rkqtk", "갑사", "갑ㅅ"),
        ("githubrkqtk", "github갑사", "github갑ㅅ"),
    ] {
        let mut session = Session::new(Mode::Auto).unwrap();
        type_keys(&mut session, keys);
        choose(&mut session, original);
        assert_eq!(session.process(Input::Backspace).preedit, deleted);
        let unchanged = session.process(Input::CycleReading);
        assert_eq!(unchanged.preedit, deleted);
        assert!(unchanged.commit.is_empty());
        assert_eq!(type_keys(&mut session, "k"), original);
        assert_eq!(session.process(Input::Finish).commit, original);
    }
}

#[test]
fn corrected_initials_can_continue_as_a_syllable_without_losing_keys() {
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "fd");
    choose(&mut session, "ㄹㅇ");
    assert_eq!(session.process(Input::Backspace).preedit, "ㄹ");
    assert_eq!(type_keys(&mut session, "k"), "라");

    session.reset();
    type_keys(&mut session, "rt");
    choose(&mut session, "ㄱㅅ");
    assert_eq!(type_keys(&mut session, "z"), "ㄱㅅㅋ");
    assert_eq!(type_keys(&mut session, "k"), "ㄱㅅ카");
    assert_eq!(session.process(Input::Backspace).preedit, "ㄱㅅㅋ");
    assert_eq!(session.process(Input::CycleReading).preedit, "ㄱㅅㅋ");
    assert_eq!(type_keys(&mut session, "k"), "ㄱㅅ카");
}

#[test]
fn deleting_a_mixed_korean_tail_restores_the_complete_latin_prefix() {
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "githubrkqtk");
    choose(&mut session, "github갑사");
    for expected in ["github갑ㅅ", "github갑", "github"] {
        assert_eq!(session.process(Input::Backspace).preedit, expected);
    }
    choose(&mut session, &compose("github"));
    choose(&mut session, "github");
}

#[test]
fn empty_fixed_finished_and_reset_sessions_do_not_rewrite_old_text() {
    for mode in [Mode::Auto, Mode::Korean, Mode::English] {
        let mut session = Session::new(mode).unwrap();
        let empty = session.process(Input::CycleReading);
        assert!(empty.consumed && empty.preedit.is_empty() && empty.commit.is_empty());
        let shown = type_keys(&mut session, "dia");
        if mode != Mode::Auto {
            assert_eq!(session.process(Input::CycleReading).preedit, shown);
        }
        session.process(Input::Finish);
        assert!(session.process(Input::CycleReading).preedit.is_empty());
        session.reset();
        assert!(session.process(Input::CycleReading).commit.is_empty());
    }
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "ai");
    choose(&mut session, "먀");
    session.reset();
    assert_eq!(type_keys(&mut session, "ai"), "ai");
    session.change_mode(Mode::English);
    assert_eq!(type_keys(&mut session, "gksrmf"), "gksrmf");
    session.change_mode(Mode::Auto);
    assert_eq!(type_keys(&mut session, "gksrmf"), "한글");
}

#[test]
fn manual_input_keeps_the_existing_bounded_token_contract() {
    let mut session = Session::new(Mode::Auto).unwrap();
    type_keys(&mut session, "ai");
    choose(&mut session, "먀");
    let shown = type_keys(&mut session, &"k".repeat(62));
    let update = session.process(Input::Letter(b'a'));
    assert_eq!(update.commit, shown);
    assert_eq!(update.preedit, "a");
    assert!(update.commit.len() < 1024);
}

fn choose(session: &mut Session, expected: &str) {
    for _ in 0..3 {
        let update = session.process(Input::CycleReading);
        assert!(update.commit.is_empty() && update.consumed);
        if update.preedit == expected {
            return;
        }
    }
    panic!("manual candidates did not include {expected}");
}

fn type_keys(session: &mut Session, keys: &str) -> String {
    let mut shown = String::new();
    for key in keys.bytes() {
        shown = session.process(Input::Letter(key)).preedit;
    }
    shown
}

fn compose(keys: &str) -> String {
    let mut composer = HangulComposer::new().unwrap();
    let mut text = String::new();
    let mut preedit = String::new();
    for key in keys.bytes() {
        let update = composer.push(key);
        text.push_str(&update.settled);
        preedit = update.preedit;
    }
    text + &preedit
}
