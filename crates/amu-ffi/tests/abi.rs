use amu_ffi::{AmuOutput, amu_session_free, amu_session_new, amu_session_process};
use std::mem::MaybeUninit;

#[test]
fn c_boundary_returns_owned_utf8_with_explicit_lengths() {
    let session = amu_session_new(1);
    assert!(!session.is_null());
    let mut output = MaybeUninit::<AmuOutput>::uninit();
    // SAFETY: this test owns the live handle and a disjoint output allocation.
    unsafe {
        for key in b"rkqtk" {
            assert_eq!(
                amu_session_process(session, 0, (*key).into(), output.as_mut_ptr()),
                0
            );
        }
        let output = output.assume_init();
        assert_eq!(output.consumed, 1);
        assert_eq!(
            &output.preedit[..output.preedit_len as usize],
            "갑사".as_bytes()
        );
        assert_eq!(output.commit_len, 0);
        amu_session_free(session);
    }
}

#[test]
fn c_boundary_reports_and_commits_a_mixed_reading() {
    let session = amu_session_new(0);
    assert!(!session.is_null());
    let mut output = MaybeUninit::<AmuOutput>::uninit();
    // SAFETY: the handle and output storage are exclusively owned by this test.
    unsafe {
        for key in b"tkatjdsdi" {
            assert_eq!(
                amu_session_process(session, 0, (*key).into(), output.as_mut_ptr()),
                0
            );
        }
        let marked = output.assume_init_ref();
        assert_eq!(marked.language, 2);
        assert_eq!(
            &marked.preedit[..marked.preedit_len as usize],
            "삼성sdi".as_bytes()
        );
        assert_eq!(amu_session_process(session, 2, 0, output.as_mut_ptr()), 0);
        let committed = output.assume_init_ref();
        assert_eq!(committed.language, 2);
        assert_eq!(committed.preedit_len, 0);
        assert_eq!(
            &committed.commit[..committed.commit_len as usize],
            "삼성sdi".as_bytes()
        );
        amu_session_free(session);
    }
}

#[test]
fn word_boundary_retains_context_but_finish_discards_it() {
    let session = amu_session_new(0);
    assert!(!session.is_null());
    let mut output = MaybeUninit::<AmuOutput>::uninit();
    // SAFETY: this test owns both allocations and reads output only after success.
    unsafe {
        for key in b"gksrmf" {
            assert_eq!(
                amu_session_process(session, 0, (*key).into(), output.as_mut_ptr()),
                0
            );
        }
        assert_eq!(amu_session_process(session, 4, 0, output.as_mut_ptr()), 0);
        assert_eq!(output.assume_init_ref().consumed, 0);
        for key in b"go" {
            assert_eq!(
                amu_session_process(session, 0, (*key).into(), output.as_mut_ptr()),
                0
            );
        }
        let korean = output.assume_init_ref();
        assert_eq!(
            &korean.preedit[..korean.preedit_len as usize],
            "해".as_bytes()
        );
        assert_eq!(amu_session_process(session, 2, 0, output.as_mut_ptr()), 0);
        for key in b"go" {
            assert_eq!(
                amu_session_process(session, 0, (*key).into(), output.as_mut_ptr()),
                0
            );
        }
        let english = output.assume_init_ref();
        assert_eq!(&english.preedit[..english.preedit_len as usize], b"go");
        amu_session_free(session);
    }
}

#[test]
fn invalid_arguments_do_not_consume_pending_composition() {
    assert!(amu_session_new(99).is_null());
    let session = amu_session_new(1);
    let mut output = MaybeUninit::<AmuOutput>::uninit();
    // SAFETY: null inputs are supported errors; other pointers are valid exclusive allocations.
    unsafe {
        assert_eq!(
            amu_session_process(std::ptr::null_mut(), 0, 114, output.as_mut_ptr()),
            1
        );
        assert_eq!(amu_session_process(session, 0, 114, output.as_mut_ptr()), 0);
        assert_eq!(
            amu_session_process(session, 0, 0x110000, output.as_mut_ptr()),
            1
        );
        assert_eq!(amu_session_process(session, 3, 99, output.as_mut_ptr()), 1);
        assert_eq!(amu_session_process(session, 2, 0, output.as_mut_ptr()), 0);
        let output = output.assume_init();
        assert_eq!(
            &output.commit[..output.commit_len as usize],
            "ㄱ".as_bytes()
        );
        amu_session_free(session);
    }
}

#[test]
fn output_layout_matches_the_public_c_header() {
    assert_eq!(std::mem::size_of::<AmuOutput>(), 2064);
    assert_eq!(std::mem::align_of::<AmuOutput>(), 4);
}
