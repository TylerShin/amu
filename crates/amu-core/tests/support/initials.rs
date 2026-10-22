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
    ("sw", "ㄴㅈ"),
    ("sg", "ㄴㅎ"),
    ("fr", "ㄹㄱ"),
    ("fq", "ㄹㅂ"),
    ("ft", "ㄹㅅ"),
    ("fx", "ㄹㅌ"),
    ("fv", "ㄹㅍ"),
    ("fg", "ㄹㅎ"),
    ("fdfd", "ㄹㅇㄹㅇ"),
    ("dwdw", "ㅇㅈㅇㅈ"),
    ("rtrt", "ㄱㅅㄱㅅ"),
    ("fdzz", "ㄹㅇㅋㅋ"),
    ("dwggg", "ㅇㅈㅎㅎㅎ"),
    ("rtzzgg", "ㄱㅅㅋㅋㅎㅎ"),
];

pub const LATIN_COUNTEREXAMPLES: &[&str] = &[
    "ai", "vi", "qi", "io", "xy", "as", "at", "we", "be", "add", "args", "test", "qzx", "vtr",
    "bld", "css", "msg", "sdk", "tcp", "fallback", "switch", "fa", "de", "re",
];
