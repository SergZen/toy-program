# mini-grep

A tiny, dependency-free grep-like CLI search utility written in idiomatic Rust
(standard library only), kept under 200 lines including tests.

## Usage

```text
mini-grep <pattern> <file_path> [--ignore-case|-i]
```

- `<pattern>` — plain substring to search for (not a regex)
- `<file_path>` — file to search
- `-i`, `--ignore-case` — optional case-insensitive matching (Unicode-aware)

Every matching line is printed as:

```text
<line_number>: <text with matches highlighted>
```

Each match is wrapped in ANSI bold red: `\x1b[1;31m…\x1b[0m`.

## Build & run

```bash
cargo build --release
./target/release/mini-grep "pattern" path/to/file.txt
./target/release/mini-grep "мир" path/to/file.txt -i
```

## Exit codes

| Code | Meaning                          |
|------|----------------------------------|
| 0    | success                          |
| 1    | file open/read (I/O) error       |
| 2    | missing/invalid CLI arguments    |

## Design notes

- The file is streamed line by line, so memory usage stays bounded.
- Matching operates on `char` boundaries, so highlighting Cyrillic, emoji,
  or any UTF-8 text never panics on slicing.
- Invalid UTF-8 bytes are decoded lossily instead of crashing.
- No `panic!`/`unwrap()` in program code: errors are reported with context.
- An empty pattern matches every line, like `grep`.

## Tests

```bash
cargo test
```
