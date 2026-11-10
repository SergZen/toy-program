//! mini-grep: a tiny grep-like line search utility.
//!
//! Usage: mini-grep <pattern> <file_path> [--ignore-case|-i]
//!
//! Prints every line containing `pattern` as `<line_number>: <text>`, wrapping
//! each match in ANSI bold-red escapes. Matches are located on `char`
//! boundaries, so searching non-ASCII text (Cyrillic, emoji, ...) is UTF-8 safe.

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::ExitCode;

const USAGE: &str = "Usage: mini-grep <pattern> <file_path> [--ignore-case|-i]";
const BOLD_RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

/// Parsed command-line configuration.
struct Config {
    pattern: String,
    path: String,
    ignore_case: bool,
}

impl Config {
    /// Parse `<pattern> <file_path>` plus optional `-i` / `--ignore-case`.
    fn parse(args: &[String]) -> Result<Config, String> {
        let (mut pattern, mut path, mut ignore_case) = (None, None, false);
        for arg in args {
            match arg.as_str() {
                "-i" | "--ignore-case" => ignore_case = true,
                s if s.starts_with('-') && s.len() > 1 => {
                    return Err(format!("unknown option '{s}'"))
                }
                s if pattern.is_none() => pattern = Some(s.to_string()),
                s if path.is_none() => path = Some(s.to_string()),
                s => return Err(format!("unexpected argument '{s}'")),
            }
        }
        match (pattern, path) {
            (Some(pattern), Some(path)) => Ok(Config { pattern, path, ignore_case }),
            _ => Err("expected <pattern> and <file_path>".into()),
        }
    }
}

/// Compare two chars, optionally ignoring Unicode case.
fn char_eq(a: char, b: char, ignore_case: bool) -> bool {
    if ignore_case {
        a.to_lowercase().eq(b.to_lowercase())
    } else {
        a == b
    }
}

/// Byte ranges of the leftmost non-overlapping matches of `pattern` in `line`.
/// Every range starts/ends on a `char` boundary, so slicing stays UTF-8 safe.
fn find_matches(line: &str, pattern: &str, ignore_case: bool) -> Vec<(usize, usize)> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let pat: Vec<char> = pattern.chars().collect();
    if pat.is_empty() {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut i = 0;
    while i + pat.len() <= chars.len() {
        if (0..pat.len()).all(|j| char_eq(chars[i + j].1, pat[j], ignore_case)) {
            let last = chars[i + pat.len() - 1];
            ranges.push((chars[i].0, last.0 + last.1.len_utf8()));
            i += pat.len();
        } else {
            i += 1;
        }
    }
    ranges
}

/// Wrap the given byte ranges of `line` in ANSI bold-red escape codes.
fn highlight(line: &str, ranges: &[(usize, usize)]) -> String {
    let mut out = String::with_capacity(line.len());
    let mut pos = 0;
    for &(start, end) in ranges {
        out.push_str(&line[pos..start]);
        out.push_str(BOLD_RED);
        out.push_str(&line[start..end]);
        out.push_str(RESET);
        pos = end;
    }
    out.push_str(&line[pos..]);
    out
}

/// Stream the file line by line, printing every line that contains the pattern.
fn run(cfg: &Config) -> Result<(), String> {
    let file = File::open(&cfg.path).map_err(|e| format!("failed to open '{}': {e}", cfg.path))?;
    let mut reader = BufReader::new(file);
    let (mut buf, mut num) = (Vec::new(), 0);
    loop {
        buf.clear();
        let bytes = reader
            .read_until(b'\n', &mut buf)
            .map_err(|e| format!("failed to read '{}': {e}", cfg.path))?;
        if bytes == 0 {
            return Ok(());
        }
        num += 1;
        let raw = String::from_utf8_lossy(&buf);
        let line = raw.trim_end_matches(|c| c == '\n' || c == '\r');
        let ranges = find_matches(line, &cfg.pattern, cfg.ignore_case);
        if cfg.pattern.is_empty() || !ranges.is_empty() {
            println!("{num}: {}", highlight(line, &ranges));
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = match Config::parse(&args) {
        Ok(cfg) => cfg,
        Err(msg) => {
            eprintln!("mini-grep: {msg}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(&cfg) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("mini-grep: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_matches_with_byte_offsets() {
        assert_eq!(find_matches("hello world", "o w", false), vec![(4, 7)]);
        assert_eq!(find_matches("aaaa", "aa", false), vec![(0, 2), (2, 4)]);
        assert!(find_matches("hello", "xyz", false).is_empty());
    }

    #[test]
    fn case_sensitive_by_default() {
        assert!(find_matches("Rust is fun", "rust", false).is_empty());
        assert_eq!(find_matches("Rust is fun", "Rust", false), vec![(0, 4)]);
    }

    #[test]
    fn case_insensitive_matches_unicode() {
        assert_eq!(find_matches("Grüße", "grüße", true), vec![(0, 7)]);
        assert_eq!(find_matches("Привет МИР", "мир", true), vec![(13, 19)]);
        assert!(find_matches("Grüße", "GRÜSSE", false).is_empty());
    }

    #[test]
    fn multibyte_matches_are_utf8_safe() {
        let line = "🙂привет";
        assert_eq!(find_matches(line, "привет", false), vec![(4, 16)]);
        assert_eq!(highlight(line, &[(4, 16)]), format!("🙂{BOLD_RED}привет{RESET}"));
    }

    #[test]
    fn highlights_with_ansi_codes() {
        assert_eq!(highlight("ab cd", &[(0, 2)]), format!("{BOLD_RED}ab{RESET} cd"));
        assert_eq!(
            highlight("x y x", &[(0, 1), (4, 5)]),
            format!("{BOLD_RED}x{RESET} y {BOLD_RED}x{RESET}")
        );
        assert_eq!(highlight("plain", &[]), "plain");
    }

    #[test]
    fn empty_pattern_has_no_ranges() {
        assert!(find_matches("anything", "", false).is_empty());
    }

    #[test]
    fn parses_flags_and_positionals() {
        let cfg = Config::parse(&["мир".into(), "notes.txt".into(), "-i".into()]).unwrap();
        assert_eq!((cfg.pattern.as_str(), cfg.path.as_str()), ("мир", "notes.txt"));
        assert!(cfg.ignore_case);
        let cfg = Config::parse(&["--ignore-case".into(), "p".into(), "f".into()]).unwrap();
        assert!(cfg.ignore_case);
        assert!(Config::parse(&["only".into()]).is_err());
        assert!(Config::parse(&[]).is_err());
        assert!(Config::parse(&["a".into(), "b".into(), "c".into()]).is_err());
    }
}
