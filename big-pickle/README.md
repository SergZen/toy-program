# mini-grep

A tiny, dependency-free grep-like CLI search utility written in under 200 lines
of idiomatic Rust (including tests).

## Usage

```text
mini-grep <pattern> <file_path> [--ignore-case|-i]
```

- `<pattern>` — substring to search for (plain substring, not a regex)
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

## Example

Given `sample.txt`:

```text
Hello, world!
Привет, МИР
🙂 emoji мир line
```

```console
$ mini-grep мир sample.txt -i
2: Привет, МИР
3: 🙂 emoji мир line
```

(matches appear in bold red in a terminal)

## Exit codes

| Code | Meaning                          |
|------|----------------------------------|
| 0    | success                          |
| 1    | file open/read (I/O) error       |
| 2    | missing/invalid CLI arguments    |

## Design notes

- The file is streamed line by line (`BufRead::read_until`), so memory usage
  stays bounded regardless of file size.
- Matching operates on `char` boundaries (`char_indices`), so highlighting
  Cyrillic, emoji, or any UTF-8 text never panics on slicing.
- Lines containing invalid UTF-8 bytes are decoded lossily instead of crashing.
- No `panic!`/`unwrap()` in program code: all errors are reported with context
  (e.g. `failed to open 'x.txt': No such file or directory`) and a non-zero
  exit code; bad arguments print usage instructions.
- An empty pattern matches every line, like `grep`.

## Tests

```bash
cargo test
```

Unit tests cover match offsets, non-overlapping matching, case-sensitive and
case-insensitive search, Unicode safety (Cyrillic, umlauts, emoji), ANSI
highlight formatting, and CLI argument parsing.
