use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::exit;

/// Shown whenever arguments are missing or invalid.
const USAGE: &str = "Usage: mini-grep <pattern> <file_path> [--ignore-case|-i]";

/// Bold-red ANSI markers used for highlighting matches.
const BOLD_RED_ON: &str = "\x1b[1;31m";
const BOLD_RED_OFF: &str = "\x1b[0m";

struct Config {
    pattern: String,
    path: String,
    ignore_case: bool,
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let (mut pattern, mut path, mut ignore_case) = (None, None, false);
    for arg in args {
        match arg.as_str() {
            "-i" | "--ignore-case" => ignore_case = true,
            _ if arg.starts_with('-') => return Err(format!("unknown flag: {arg}\n{USAGE}")),
            _ if pattern.is_none() => pattern = Some(arg.clone()),
            _ if path.is_none() => path = Some(arg.clone()),
            _ => return Err(format!("too many arguments\n{USAGE}")),
        }
    }
    match (pattern, path) {
        (Some(p), Some(f)) if !p.is_empty() => Ok(Config {
            pattern: p,
            path: f,
            ignore_case,
        }),
        (Some(_), Some(_)) => Err(format!("empty pattern\n{USAGE}")),
        _ => Err(format!("missing arguments\n{USAGE}")),
    }
}

/// Lowercases one character. The 1:1 mapping keeps folded text index-aligned
/// with the original characters, so no byte-offset remapping is ever needed.
fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Byte ranges of every non-overlapping occurrence of `pattern` in `line`.
/// Slicing only ever happens at `char` boundaries, so any UTF-8 input
/// (Cyrillic, emoji, accents, ...) is handled without panicking.
fn find_spans(line: &str, pattern: &str, ignore_case: bool) -> Vec<(usize, usize)> {
    if !ignore_case {
        return line
            .match_indices(pattern)
            .map(|(i, m)| (i, i + m.len()))
            .collect();
    }
    let folded: Vec<char> = line.chars().map(fold).collect();
    let starts: Vec<usize> = line.char_indices().map(|(i, _)| i).collect();
    let needle: Vec<char> = pattern.chars().map(fold).collect();
    let mut spans = Vec::new();
    if needle.is_empty() {
        return spans;
    }
    let mut from = 0;
    loop {
        let Some(win) = folded[from..]
            .windows(needle.len())
            .position(|w| w == needle.as_slice())
        else {
            break;
        };
        let last = from + win + needle.len() - 1;
        spans.push((starts[from + win], starts[last] + folded[last].len_utf8()));
        from += win + needle.len();
    }
    spans
}

/// Wraps every matched byte range in bold-red ANSI codes.
fn highlight(line: &str, spans: &[(usize, usize)]) -> String {
    let mut out = String::new();
    let mut at = 0;
    for &(start, end) in spans {
        if start < at {
            continue;
        }
        out.push_str(&line[at..start]);
        out.push_str(BOLD_RED_ON);
        out.push_str(&line[start..end]);
        out.push_str(BOLD_RED_OFF);
        at = end;
    }
    out.push_str(&line[at..]);
    out
}

/// Streams the file line by line; returns the number of matching lines.
fn run(config: &Config) -> Result<usize, String> {
    let file = File::open(&config.path)
        .map_err(|e| format!("mini-grep: cannot open '{}': {e}", config.path))?;
    let reader = BufReader::new(file);
    let mut count = 0;
    for (number, line) in reader.lines().enumerate() {
        let line =
            line.map_err(|e| format!("mini-grep: I/O error in '{}': {e}", config.path))?;
        let spans = find_spans(&line, &config.pattern, config.ignore_case);
        if !spans.is_empty() {
            println!("{}: {}", number + 1, highlight(&line, &spans));
            count += 1;
        }
    }
    Ok(count)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("mini-grep: {e}");
            exit(2);
        }
    };
    match run(&config) {
        Ok(n) if n > 0 => {}
        Ok(_) => exit(1), // grep-style: nothing matched
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end helper: find matches and format them exactly as printed.
    fn hl(line: &str, pattern: &str, ignore_case: bool) -> String {
        highlight(line, &find_spans(line, pattern, ignore_case))
    }

    #[test]
    fn substring_matching_finds_all_occurrences() {
        assert_eq!(
            hl("rust is fast, rust is fun", "rust", false),
            "\x1b[1;31mrust\x1b[0m is fast, \x1b[1;31mrust\x1b[0m is fun"
        );
        assert_eq!(hl("hello", "xyz", false), "hello"); // no match: unchanged
    }

    #[test]
    fn case_insensitive_matching() {
        assert_eq!(hl("Hello theRE", "theRe", true), "Hello \x1b[1;31mtheRE\x1b[0m");
        assert_eq!(hl("Привет, МИР!", "мир", true), "Привет, \x1b[1;31mМИР\x1b[0m!");
        assert_eq!(hl("Hello theRE", "theRe", false), "Hello theRE"); // sensitive
    }

    #[test]
    fn utf8_text_never_panics_or_misaligns() {
        assert_eq!(hl("héllo 🦀 café", "🦀", false), "héllo \x1b[1;31m🦀\x1b[0m café");
        assert_eq!(hl("ёжик в тумане", "в тумане", false), "ёжик \x1b[1;31mв тумане\x1b[0m");
        assert_eq!(hl("ДОНЬЕР 🦀 ждЁт", "🦀 ждёт", true), "ДОНЬЕР \x1b[1;31m🦀 ждЁт\x1b[0m");
    }

    #[test]
    fn invalid_usage_is_reported_not_panicked() {
        assert!(parse_args(&[]).is_err()); // nothing given
        assert!(parse_args(&["a".to_string()]).is_err()); // no file
        assert!(parse_args(&["a".to_string(), "b".to_string(), "c".to_string()]).is_err());
        assert!(parse_args(&["a".to_string(), "b".to_string(), "--oops".to_string()]).is_err());
        assert!(parse_args(&["".to_string(), "b".to_string()]).is_err()); // empty pattern
        let c = parse_args(&["мир".to_string(), "f.txt".to_string(), "-i".to_string()]);
        assert!(c.is_ok_and(|c| c.ignore_case && c.pattern == "мир" && c.path == "f.txt"));
    }
}
