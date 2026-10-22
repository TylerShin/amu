#include <stdio.h>
#include <string.h>
#include "hangul.h"

#define MAX_TRACE_BYTES 4096

static void write_trace(HangulInputContext *context, const char *keys);
static void write_codepoints(const ucschar *text);

int main(void) {
    HangulInputContext *context = hangul_ic_new("2");
    if (context == NULL) {
        fputs("Could not create a libhangul context\n", stderr);
        return 1;
    }

    char keys[MAX_TRACE_BYTES];
    while (fgets(keys, sizeof(keys), stdin) != NULL) {
        if (strchr(keys, '\n') == NULL) {
            fputs("Trace exceeds limit or lacks a final newline\n", stderr);
            hangul_ic_delete(context);
            return 1;
        }
        keys[strcspn(keys, "\n")] = '\0';
        hangul_ic_reset(context);
        write_trace(context, keys);
    }

    hangul_ic_delete(context);
    return ferror(stdin) || ferror(stdout) ? 1 : 0;
}

static void write_trace(HangulInputContext *context, const char *keys) {
    fputc('[', stdout);
    for (size_t i = 0; keys[i] != '\0'; i++) {
        bool consumed;
        const ucschar *commit;
        if (keys[i] == '<') {
            consumed = hangul_ic_backspace(context);
            commit = hangul_ic_get_commit_string(context);
        } else if (keys[i] == '|') {
            consumed = true;
            commit = hangul_ic_flush(context);
        } else {
            consumed = hangul_ic_process(context, (unsigned char)keys[i]);
            commit = hangul_ic_get_commit_string(context);
        }

        if (i != 0) fputc(',', stdout);
        fprintf(stdout, "{\"consumed\":%s,\"commit\":", consumed ? "true" : "false");
        write_codepoints(commit);
        fputs(",\"preedit\":", stdout);
        write_codepoints(hangul_ic_get_preedit_string(context));
        fputc('}', stdout);
    }
    fputs("]\n", stdout);
}

static void write_codepoints(const ucschar *text) {
    fputc('[', stdout);
    for (size_t i = 0; text[i] != 0; i++) {
        if (i != 0) fputc(',', stdout);
        fprintf(stdout, "%u", text[i]);
    }
    fputc(']', stdout);
}
