#![cfg(feature = "anifilebert")]

use anime_organizer::{anifilebert, AnimeFileInfo, FilenameParser};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static PATHBUF_LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"PathBuf::from\("((?:\\.|[^"\\])*)"\)"#).expect("valid path regex")
});

static PATH_LITERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"let\s+(?:path|p)\s*=\s*"((?:\\.|[^"\\])*)"\s*;"#).expect("valid path regex")
});

#[test]
#[ignore = "full AniFileBERT compatibility report; run explicitly after model updates"]
fn anifilebert_matches_rule_parser_cases() {
    let cases = parser_test_cases();
    assert!(!cases.is_empty(), "no parser test cases found");

    let total = cases.len();
    let mut failures = Vec::new();
    for case in cases {
        let expected = FilenameParser::parse(&case.path)
            .unwrap_or_else(|| panic!("rule parser failed existing parser test case: {case:?}"));
        let actual = match anifilebert::parse_path(&case.path) {
            Ok(Some(info)) => info,
            Ok(None) => {
                failures.push(format!("{}: no AniFileBERT parse", case.display()));
                continue;
            }
            Err(error) => {
                failures.push(format!(
                    "{}: AniFileBERT runtime error: {error}",
                    case.display()
                ));
                continue;
            }
        };

        if !same_parser_fields(&actual, &expected) {
            failures.push(format!(
                "{}:\n  expected {expected:?}\n    actual {actual:?}",
                case.display()
            ));
        }
    }

    if !failures.is_empty() {
        let shown = failures.iter().take(40).cloned().collect::<Vec<_>>();
        panic!(
            "AniFileBERT matched {} / {total} existing parser cases and mismatched {}; first {}:\n{}",
            total - failures.len(),
            failures.len(),
            shown.len(),
            shown.join("\n")
        );
    }
}

fn same_parser_fields(actual: &AnimeFileInfo, expected: &AnimeFileInfo) -> bool {
    actual.publisher == expected.publisher
        && actual.anime_name == expected.anime_name
        && actual.episode == expected.episode
        && actual.tags == expected.tags
        && actual.extension == expected.extension
}

#[derive(Debug)]
struct ParserCase {
    source: String,
    path: PathBuf,
}

impl ParserCase {
    fn display(&self) -> String {
        format!("{}: {}", self.source, self.path.display())
    }
}

fn parser_test_cases() -> Vec<ParserCase> {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut cases = Vec::new();

    for entry in fs::read_dir(&test_dir).expect("read tests dir") {
        let entry = entry.expect("read tests dir entry");
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !file_name.starts_with("parser_tests") || !file_name.ends_with(".rs") {
            continue;
        }

        let content = fs::read_to_string(&path).expect("read parser test source");
        for literal in path_literals(&content) {
            cases.push(ParserCase {
                source: file_name.to_string(),
                path: PathBuf::from(literal),
            });
        }
    }

    cases.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then(left.path.cmp(&right.path))
    });
    cases.dedup_by(|left, right| left.source == right.source && left.path == right.path);
    cases
}

fn path_literals(content: &str) -> Vec<String> {
    let mut literals = Vec::new();
    for captures in PATHBUF_LITERAL.captures_iter(content) {
        literals.push(unescape_rust_string(&captures[1]));
    }
    for captures in PATH_LITERAL.captures_iter(content) {
        literals.push(unescape_rust_string(&captures[1]));
    }
    literals
}

fn unescape_rust_string(value: &str) -> String {
    let mut result = String::new();
    let mut chars = value.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            result.push(ch);
            continue;
        }

        match chars.next() {
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some(other) => result.push(other),
            None => result.push('\\'),
        }
    }

    result
}
