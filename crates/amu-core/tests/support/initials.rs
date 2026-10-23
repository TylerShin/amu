// Evaluation examples only; the decoder never reads this list.
pub const EXPRESSIONS: &[(&str, &str)] = &[
    ("fd", "ㄹㅇ"),
    ("dw", "ㅇㅈ"),
    ("rt", "ㄱㅅ"),
    ("wt", "ㅈㅅ"),
    ("af", "ㅁㄹ"),
    ("gd", "ㅎㅇ"),
    ("eq", "ㄷㅂ"),
    ("tr", "ㅅㄱ"),
    ("qt", "ㅂㅅ"),
    ("fdfd", "ㄹㅇㄹㅇ"),
    ("dwdw", "ㅇㅈㅇㅈ"),
    ("rtrt", "ㄱㅅㄱㅅ"),
    ("fdzz", "ㄹㅇㅋㅋ"),
    ("dwggg", "ㅇㅈㅎㅎㅎ"),
    ("rtzzgg", "ㄱㅅㅋㅋㅎㅎ"),
];

// Real intended initials that the general corpus does not separate reliably.
// Evaluation reports these misses; they are not relabelled as intended English.
pub const UNCERTAIN_EXPRESSIONS: &[(&str, &str)] = &[("sw", "ㄴㅈ"), ("sg", "ㄴㅎ")];

pub const LATIN_COUNTEREXAMPLES: &[&str] = &[
    "vs", "cs", "tv", "st", "sf", "cf", "fs", "ds", "ai", "vi", "qi", "io", "xy", "as", "at", "we",
    "be", "add", "args", "test", "qzx", "vtr", "bld", "css", "msg", "sdk", "tcp", "fallback",
    "switch", "fa", "de", "re",
];
