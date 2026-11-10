use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

const USAGE: &str = "Usage: mini-grep <pattern> <file_path> [--ignore-case|-i]";
const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, PartialEq)]
struct Args {
    pattern: String,
    path: String,
    ignore_case: bool,
}

fn parse_args(argv: &[String]) -> Result<Option<Args>, String> {
    if argv.iter().skip(1).any(|arg| arg == "-h" || arg == "--help") {
        return Ok(None);
    }
    let mut positional = Vec::new();
    let mut ignore_case = false;
    for arg in argv.iter().skip(1) {
        match arg.as_str() {
            "--ignore-case" | "-i" => ignore_case = true,
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown option: {value}"));
            }
            value => positional.push(value.to_string()),
        }
    }
    if positional.len() != 2 {
        return Err("expected exactly two arguments".to_string());
    }
    let mut positional = positional.into_iter();
    let pattern = positional.next().ok_or("missing pattern")?;
    let path = positional.next().ok_or("missing file path")?;
    Ok(Some(Args { pattern, path, ignore_case }))
}

fn find_ranges(line: &str, pattern: &str, ignore_case: bool) -> Vec<(usize, usize)> {
    if pattern.is_empty() || line.is_empty() {
        return Vec::new();
    }
    if !ignore_case {
        return line
            .match_indices(pattern)
            .map(|(start, text)| (start, start + text.len()))
            .collect();
    }
    let needle = pattern.to_lowercase();
    let bounds: Vec<usize> = line
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(line.len()))
        .collect();
    let mut matches = Vec::new();
    let mut start = 0;
    while start + 1 < bounds.len() {
        let mut found = None;
        let mut end = start + 1;
        while end < bounds.len() {
            let candidate = line[bounds[start]..bounds[end]].to_lowercase();
            if candidate == needle {
                found = Some(end);
                break;
            }
            if candidate.len() >= needle.len() {
                break;
            }
            end += 1;
        }
        match found {
            Some(end) => {
                matches.push((bounds[start], bounds[end]));
                start = end;
            }
            None => start += 1,
        }
    }
    matches
}

fn highlight(line: &str, ranges: &[(usize, usize)]) -> String {
    let mut result = String::with_capacity(line.len() + ranges.len() * (RED.len() + RESET.len()));
    let mut cursor = 0;
    for &(start, end) in ranges {
        result.push_str(&line[cursor..start]);
        result.push_str(RED);
        result.push_str(&line[start..end]);
        result.push_str(RESET);
        cursor = end;
    }
    result.push_str(&line[cursor..]);
    result
}

fn find_matches(line: &str, pattern: &str, ignore_case: bool) -> Option<String> {
    let ranges = find_ranges(line, pattern, ignore_case);
    if ranges.is_empty() {
        None
    } else {
        Some(highlight(line, &ranges))
    }
}

fn run(args: &Args) -> Result<(), String> {
    let file = File::open(&args.path)
        .map_err(|error| format!("cannot open '{}': {error}", args.path))?;
    let reader = BufReader::new(file);
    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| format!("cannot read '{}': {error}", args.path))?;
        if let Some(rendered) = find_matches(&line, &args.pattern, args.ignore_case) {
            println!("{}: {}", index + 1, rendered);
        }
    }
    Ok(())
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    match parse_args(&argv) {
        Ok(Some(args)) => {
            if let Err(error) = run(&args) {
                eprintln!("mini-grep: {error}");
                process::exit(1);
            }
        }
        Ok(None) => println!("{USAGE}"),
        Err(error) => {
            eprintln!("mini-grep: {error}");
            eprintln!("{USAGE}");
            process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_required_and_optional_arguments() {
        let expected = Some(Args {
            pattern: "needle".to_string(),
            path: "file.txt".to_string(),
            ignore_case: true,
        });
        assert_eq!(
            parse_args(&argv(&["mini-grep", "needle", "file.txt", "-i"])),
            Ok(expected)
        );
    }

    #[test]
    fn rejects_missing_or_invalid_arguments() {
        assert!(parse_args(&argv(&["mini-grep"])).is_err());
        assert!(parse_args(&argv(&["mini-grep", "needle"])).is_err());
        assert!(parse_args(&argv(&["mini-grep", "needle", "f", "--bogus"])).is_err());
    }

    #[test]
    fn finds_case_sensitive_matches() {
        assert_eq!(
            find_ranges("one ONE one", "one", false),
            vec![(0, 3), (8, 11)]
        );
    }

    #[test]
    fn finds_case_insensitive_unicode_matches() {
        assert_eq!(
            find_ranges("Привет, МИР и мир", "мир", true),
            vec![(14, 20), (24, 30)]
        );
    }

    #[test]
    fn formats_ansi_highlighting_around_emoji() {
        let line = "a😀b";
        let ranges = find_ranges(line, "😀", false);
        assert_eq!(highlight(line, &ranges), "a\x1b[1;31m😀\x1b[0mb");
    }

    #[test]
    fn returns_only_matching_lines_with_highlights() {
        assert_eq!(find_matches("nothing here", "needle", false), None);
        assert_eq!(
            find_matches("a NEEDLE b", "needle", true),
            Some("a \x1b[1;31mNEEDLE\x1b[0m b".to_string())
        );
    }
}
