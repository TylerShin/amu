#ifndef AMU_H
#define AMU_H
#include <stdint.h>

typedef struct AmuSession AmuSession;
typedef struct {
    uint32_t consumed;
    uint32_t language; /* 0 English, 1 Korean, 2 mixed */
    uint32_t commit_len;
    uint32_t preedit_len;
    uint8_t commit[1024];
    uint8_t preedit[1024];
} AmuOutput;

/* Modes: 0 Auto, 1 Korean, 2 English. NULL means creation failed. */
AmuSession *amu_session_new(uint32_t mode);
void amu_session_free(AmuSession *session);
/* Kinds: 0 ASCII letter, 1 Backspace, 2 Finish (clear context), 3 change mode,
   4 word boundary (commit and remember language; caller inserts the space).
   Returns: 0 success, 1 invalid arguments, 2 engine failure.
   Output is valid only on success. Strings are length-delimited UTF-8.
   After engine failure, discard the handle. Calls must be exclusive and occur
   on the creating thread. No callback or pointer into Rust is returned. */
uint32_t amu_session_process(AmuSession *session, uint32_t kind, uint32_t value, AmuOutput *output);
uint32_t amu_session_reset(AmuSession *session);
#endif
