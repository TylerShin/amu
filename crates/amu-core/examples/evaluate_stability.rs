#[path = "../tests/support/keyboard.rs"]
mod keyboard;
#[path = "../tests/support/typing_trace.rs"]
mod typing_trace;

use std::collections::BTreeMap;
use typing_trace::{Metrics, automatic, intended};

fn main() {
    let mut arguments = std::env::args().skip(1);
    let input = arguments
        .next()
        .expect("evaluate_stability <intent.tsv> [trace.tsv] | <--korean|--english> <corpus.conllu> [trace.tsv]");
    let source = match input.as_str() {
        "--korean" | "--english" => corpus_intents(&input, &arguments.next().expect("corpus path")),
        _ => std::fs::read_to_string(input).expect("intent fixtures"),
    };
    let snapshot_path = arguments.next();
    let mut results = BTreeMap::<String, Metrics>::new();
    let mut snapshot = String::from("category\tcontext\tkeys\texpected\tactual\tframes\n");
    for line in source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            3,
            "category, expected text, intended segments"
        );
        let expected = intended(fields[2]);
        assert_eq!(expected.committed, fields[1], "invalid intent fixture");
        for (name, context) in [
            ("neutral", ""),
            ("Korean", "gksrmf gksrmf gksrmf gksrmf"),
            ("English", "hello hello hello hello"),
        ] {
            let actual = automatic(&expected.keys, context);
            assert_eq!(
                actual.frames.last().unwrap(),
                &actual.committed,
                "commit must equal the last visible frame"
            );
            results
                .entry(fields[0].to_owned())
                .or_default()
                .observe(&expected, &actual);
            snapshot.push_str(&format!(
                "{}\t{name}\t{}\t{}\t{}\t{}\n",
                fields[0],
                expected.keys,
                expected.committed,
                actual.committed,
                actual.frames.join("|")
            ));
        }
    }
    println!("category\tfinal\tcorrect frames\trelabeled keys\toscillating keys\tmean stable key");
    for (name, metrics) in results {
        println!(
            "{name}\t{}/{}\t{}/{}\t{}\t{}\t{:.2}",
            metrics.correct,
            metrics.tokens,
            metrics.correct_frames,
            metrics.frames,
            metrics.relabeled_keys,
            metrics.oscillating_keys,
            metrics.stable_at as f64 / metrics.tokens as f64
        );
    }
    if let Some(path) = snapshot_path {
        std::fs::write(path, snapshot).expect("write requested public-fixture trace");
    }
}

fn corpus_intents(language: &str, path: &str) -> String {
    let source = std::fs::read_to_string(path).expect("public corpus");
    let mut intents = String::new();
    let mut english = std::collections::BTreeSet::new();
    for line in source.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 10 || fields[0].parse::<usize>().is_err() {
            continue;
        }
        let text = fields[1];
        if language == "--korean"
            && !text.is_empty()
            && text.chars().all(|c| ('가'..='힣').contains(&c))
        {
            let keys = keyboard::korean_keys(text);
            assert!(keys.len() <= 64, "corpus token exceeds trace limit");
            intents.push_str(&format!("Korean\t{text}\tk:{keys}\n"));
        } else if language == "--english"
            && (4..=64).contains(&text.len())
            && text.bytes().all(|key| key.is_ascii_alphabetic())
        {
            english.insert(text.to_ascii_lowercase());
        }
    }
    for word in english {
        intents.push_str(&format!("English\t{word}\te:{word}\n"));
    }
    intents
}
