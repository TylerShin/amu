// Shared by regression tests and the before/after evaluator, never by the model.
pub const UNREGISTERED_PREFIXES: &[&str] = &[
    "ai", "Ai", "aI", "AI", "vi", "qi", "xy", "io", "zq", "florp",
];
pub const KOREAN_SUFFIXES: &[(&str, &str)] = &[
    ("rh", "고"),
    ("rk", "가"),
    ("fh", "로"),
    ("fmf", "를"),
    ("dprp", "에게"),
    ("qnxj", "부터"),
    ("dptj", "에서"),
    ("gksrmf", "한글"),
    ("rkqtk", "갑사"),
];
pub const KOREAN_WORDS: &[(&str, &str)] = &[
    ("dkdlrh", "아이고"),
    ("gkrh", "하고"),
    ("rkrh", "가고"),
    ("dlfh", "이로"),
    ("skqnxj", "나부터"),
    ("dndmlfmf", "우의를"),
    ("sodptj", "내에서"),
    ("sodprp", "내에게"),
    ("vjarh", "펌고"),
    ("znrrh", "쿡고"),
    ("rOrk", "걔가"),
    ("dOrk", "얘가"),
    ("djEjfRK", "어떨까"),
    ("chachagl", "촘촘히"),
];
pub const ENGLISH_WORDS: &[&str] = &[
    "hello", "github", "florp", "quirk", "quick", "oxygen", "aims", "vivid", "violin", "violet",
    "quiet", "oxide", "hybrid", "system",
];

// Held out from English word hints. Their spelling must survive both word
// completion and a later, genuine Korean suffix.
pub const CONNECTED_ENGLISH_WORDS: &[&str] = &[
    "fallback", "callback", "rollback", "stack", "black", "track", "block", "switch", "catch",
    "fetch", "scratch", "watch", "patch", "sketch", "dispatch", "packet",
];
