# mini-grep

A tiny `grep`-style search utility in idiomatic Rust — the complete codebase,
tests included, fits in **under 200 lines** (`src/main.rs`).

## Usage

```
mini-grep <pattern> <file_path> [--ignore-case|-i]
```

| Argument        | Description                              |
|-----------------|------------------------------------------|
| `<pattern>`     | Substring to search for (must not be empty) |
| `<file_path>`   | File to search                            |
| `--ignore-case` / `-i` | Case-insensitive matching (optional) |

Every matching line is printed as:

```
<line_number>: <highlighted_text>
```

Matches inside the line are wrapped in bold-red ANSI escapes
(`\x1b[1;31m...\x1b[0m`).

### Examples

```
$ mini-grep error app.log
42: 2024-06-01 13:12:44 ERROR disk almost full

$ mini-grep мир war-and-peace.txt -i
1023: И сказала она: мир тебе, князь!
```

### Exit codes

| Code | Meaning                              |
|------|--------------------------------------|
| `0`  | At least one line matched            |
| `1`  | No matches (`grep`-compatible)       |
| `2`  | Bad arguments or file/I-O error      |

## Features & guarantees

- **Streaming reads** — the file is read line by line through a `BufReader`,
  so arbitrarily large files never load fully into memory.
- **UTF-8 safe matching & highlighting** — case-sensitive search uses
  `str::match_indices`, and case-insensitive search folds characters 1:1 and
  only ever slices at `char` boundaries. Text containing Cyrillic, emoji,
  accents, etc. is never mis-sliced and never panics.
- **Graceful errors** — missing/invalid arguments print clear usage text;
  unreadable files and I-O failures print a custom message. There are no
  `unwrap()`/`expect()`/`panic!` calls outside of ordinary `Result` handling.
- **Highlight formatting** — matched substrings are re-sliced from the
  original text, so the printed line preserves the original characters.

### Known limitation

Case-insensitive folding is per-character (`char::to_lowercase`), so the rare
mappings where one char lowercases to *two* (e.g. `İ` → `i̇`, `ẞ` → `ss`) are
folded to their first lowercase char (`İ` → `i`).

## Development

```
cargo build            # build the binary
cargo test             # run the unit tests
cargo run -- <pattern> <file> [-i]
```

## Project layout

| File         | Purpose                                  |
|--------------|------------------------------------------|
| `src/main.rs`| All logic + unit tests (`#[cfg(test)]`)  |
| `Cargo.toml` | Package manifest (no dependencies)       |
