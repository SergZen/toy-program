# mini-grep

A small, dependency-free command-line search utility written in Rust. It reads a
file line by line, prints matching lines with their line numbers, and highlights
every match in bold red.

## Build

```sh
cargo build --release
```

The binary is written to `target/release/mini-grep`.

## Usage

```text
mini-grep <pattern> <file_path> [--ignore-case|-i]
```

- `<pattern>` — literal substring to search for (not a regular expression).
- `<file_path>` — file to search.
- `--ignore-case`, `-i` — perform a case-insensitive search.
- `-h`, `--help` — print usage information.

### Examples

```sh
# Case-sensitive search
mini-grep "TODO" notes.txt

# Case-insensitive search
mini-grep "todo" notes.txt -i

# Unicode text works too
mini-grep "мир" article.txt
```

Example output:

```text
3: fix the \x1b[1;31mTODO\x1b[0m marker
7: TODO: write documentation
```

In a terminal the matched text is displayed in bold red.

## Behavior and safety

- Files are streamed with `BufReader` and processed one line at a time.
- ANSI highlighting is inserted only at valid UTF-8 character boundaries, so
  non-ASCII text, Cyrillic, and emoji are handled without panics.
- Missing files, unreadable files, invalid UTF-8, and bad command-line arguments
  produce a clear `mini-grep: ...` error on stderr and a non-zero exit status.
- Missing or invalid arguments also print the usage line.
- Empty patterns match nothing.

## Exit status

| Status | Meaning |
| ------ | ------- |
| `0`    | Search completed, or help was requested |
| `1`    | File could not be opened or read |
| `2`    | Invalid command-line arguments |

## Tests

```sh
cargo test
```

The unit tests cover argument parsing, case-sensitive and case-insensitive
matching, Unicode/Cyrillic matching, emoji-safe slicing, ANSI formatting, and
line filtering.
