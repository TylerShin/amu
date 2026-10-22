use std::{
    ffi::{c_int, c_void},
    marker::PhantomData,
    ptr::NonNull,
    rc::Rc,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ComposerError {
    AllocationFailed,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Composition {
    pub consumed: bool,
    pub settled: String,
    pub preedit: String,
}

/// One libhangul context, owned and used by one input session on one thread.
pub struct HangulComposer {
    context: NonNull<c_void>,
    _single_thread: PhantomData<Rc<()>>,
}

impl HangulComposer {
    pub fn new() -> Result<Self, ComposerError> {
        // SAFETY: the static keyboard ID is NUL-terminated. We own the returned context.
        let context = NonNull::new(unsafe { hangul_ic_new(c"2".as_ptr()) })
            .ok_or(ComposerError::AllocationFailed)?;
        Ok(Self {
            context,
            _single_thread: PhantomData,
        })
    }

    pub fn push(&mut self, ascii: u8) -> Composition {
        // SAFETY: context is live, exclusively borrowed, and accepts an ASCII key code.
        let consumed = unsafe { hangul_ic_process(self.context.as_ptr(), ascii.into()) };
        self.read_composition(consumed)
    }

    pub fn backspace(&mut self) -> Composition {
        // SAFETY: context is live and exclusively borrowed.
        let consumed = unsafe { hangul_ic_backspace(self.context.as_ptr()) };
        self.read_composition(consumed)
    }

    pub fn reset(&mut self) {
        // SAFETY: reset retains ownership of the live context.
        unsafe { hangul_ic_reset(self.context.as_ptr()) };
    }

    fn read_composition(&self, consumed: bool) -> Composition {
        // SAFETY: libhangul owns both NUL-terminated buffers until the next mutation.
        // Copy before returning so callers never retain C pointers.
        unsafe {
            Composition {
                consumed,
                settled: copy_ucs4(hangul_ic_get_commit_string(self.context.as_ptr())),
                preedit: copy_ucs4(hangul_ic_get_preedit_string(self.context.as_ptr())),
            }
        }
    }
}

impl Drop for HangulComposer {
    fn drop(&mut self) {
        // SAFETY: this context is uniquely owned and released exactly once.
        unsafe { hangul_ic_delete(self.context.as_ptr()) };
    }
}

unsafe fn copy_ucs4(text: *const u32) -> String {
    let mut result = String::new();
    for offset in 0..64 {
        // SAFETY: libhangul's documented context buffer has 64 UCS-4 slots.
        let codepoint = unsafe { *text.add(offset) };
        if codepoint == 0 {
            return result;
        }
        result.push(char::from_u32(codepoint).expect("libhangul returned invalid Unicode"));
    }
    panic!("libhangul returned an unterminated composition buffer");
}

unsafe extern "C" {
    fn hangul_ic_new(keyboard: *const std::ffi::c_char) -> *mut c_void;
    fn hangul_ic_delete(context: *mut c_void);
    fn hangul_ic_process(context: *mut c_void, ascii: c_int) -> bool;
    fn hangul_ic_backspace(context: *mut c_void) -> bool;
    fn hangul_ic_reset(context: *mut c_void);
    fn hangul_ic_get_commit_string(context: *mut c_void) -> *const u32;
    fn hangul_ic_get_preedit_string(context: *mut c_void) -> *const u32;
}
