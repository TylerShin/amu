use amu_core::{Input, Language, Mode, Session, Update};
use std::panic::{AssertUnwindSafe, catch_unwind};

const BUFFER_SIZE: usize = 1024;
const OK: u32 = 0;
const INVALID_ARGUMENT: u32 = 1;
const ENGINE_FAILURE: u32 = 2;

#[repr(C)]
pub struct AmuOutput {
    pub consumed: u32,
    pub language: u32,
    pub commit_len: u32,
    pub preedit_len: u32,
    pub commit: [u8; BUFFER_SIZE],
    pub preedit: [u8; BUFFER_SIZE],
}

#[unsafe(no_mangle)]
pub extern "C" fn amu_session_new(mode: u32) -> *mut Session {
    let Some(mode) = parse_mode(mode) else {
        return std::ptr::null_mut();
    };
    catch_unwind(|| {
        Session::new(mode)
            .ok()
            .map_or(std::ptr::null_mut(), |session| {
                Box::into_raw(Box::new(session))
            })
    })
    .unwrap_or(std::ptr::null_mut())
}

/// # Safety
/// `session` must be null or a live handle from amu_session_new, released once.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amu_session_free(session: *mut Session) {
    if !session.is_null() {
        // SAFETY: the caller transfers ownership of this live handle back to Rust.
        drop(unsafe { Box::from_raw(session) });
    }
}

/// # Safety
/// Handle must be live, exclusively accessed on its creating thread; output must
/// be a writable, aligned AmuOutput disjoint from the handle. Never retain aliases.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amu_session_process(
    session: *mut Session,
    kind: u32,
    value: u32,
    output: *mut AmuOutput,
) -> u32 {
    if session.is_null() || output.is_null() {
        return INVALID_ARGUMENT;
    }
    let input = match kind {
        0 if value <= 127 && (value as u8).is_ascii_alphabetic() => Input::Letter(value as u8),
        1 => Input::Backspace,
        2 => Input::Finish,
        4 => Input::WordBoundary,
        3 => {
            let Some(mode) = parse_mode(value) else {
                return INVALID_ARGUMENT;
            };
            return write_update(session, output, |session| session.change_mode(mode));
        }
        _ => return INVALID_ARGUMENT,
    };
    write_update(session, output, |session| session.process(input))
}

/// # Safety
/// Same handle ownership and thread rules as amu_session_process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amu_session_reset(session: *mut Session) -> u32 {
    if session.is_null() {
        return INVALID_ARGUMENT;
    }
    match catch_unwind(AssertUnwindSafe(|| unsafe { &mut *session }.reset())) {
        Ok(()) => OK,
        Err(_) => ENGINE_FAILURE,
    }
}

fn write_update(
    session: *mut Session,
    output: *mut AmuOutput,
    operation: impl FnOnce(&mut Session) -> Update,
) -> u32 {
    let result = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: all public entry points validate non-null and require exclusive access.
        let update = operation(unsafe { &mut *session });
        let mut encoded = AmuOutput {
            consumed: u32::from(update.consumed),
            language: match update.language {
                Language::English => 0,
                Language::Korean => 1,
                Language::Mixed => 2,
            },
            commit_len: update.commit.len() as u32,
            preedit_len: update.preedit.len() as u32,
            commit: [0; BUFFER_SIZE],
            preedit: [0; BUFFER_SIZE],
        };
        encoded.commit[..update.commit.len()].copy_from_slice(update.commit.as_bytes());
        encoded.preedit[..update.preedit.len()].copy_from_slice(update.preedit.as_bytes());
        // SAFETY: output is caller-owned writable storage of the declared C layout.
        unsafe { output.write(encoded) };
    }));
    match result {
        Ok(()) => OK,
        Err(_) => ENGINE_FAILURE,
    }
}

fn parse_mode(mode: u32) -> Option<Mode> {
    match mode {
        0 => Some(Mode::Auto),
        1 => Some(Mode::Korean),
        2 => Some(Mode::English),
        _ => None,
    }
}
