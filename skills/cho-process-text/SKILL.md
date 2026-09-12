---
name: cho-process-text
description: Build and verify cho shell one-liners. Use for cho commands or text pipelines that need typed field comparisons and conversions.
---

# Process text with cho

cho evaluates small, composable Lisp-like expressions once per input record.
Fields are strings; functions convert them to the types their signatures need.
Malformed typed input produces an error rather than silently failing a match.
Keep sorting and cross-record aggregation in other pipeline tools.

## Discover the available syntax

Check syntax against the executable you will actually use. Use `cho -k QUERY`
to find functions and `cho --help FUNCTION` for signatures, examples, and notes.
For example, `cho -k trim` finds names and `cho --help s/trim` explains usage.
Use full `cho --help` when input options or general language rules are needed;
`cho -k` lists all function and form names.
These discovery commands run separately from execution options and programs.
Do not assume a function exists from its name or from another installed version.

If cho is unavailable, say so and distinguish a proposed command from one you
have verified. In a cho checkout, use `cargo run --quiet --` or build and use
`target/debug/cho` to check the local implementation. Installing or upgrading
cho is not a prerequisite for merely explaining a command.

## Build the command

- Inspect representative input for delimiters, headers, quoting, empty fields,
  and column meanings. Choose default whitespace splitting, regex `-F`,
  `--csv`, or `--tsv` accordingly. Use `--skip-header` for CSV or TSV headers.
- Quote the program with single shell quotes so the shell preserves `$1` and
  other field references. Regex literals preserve backslashes; quoted cho
  strings require doubled backslashes. Consult help when patterns contain `/`.
- Compose values and predicates with the required type: numeric comparisons
  use `>`, `=`, etc.; strings use `s/`, calendar dates `d/`, timestamps `dt/`.
  Check signatures for other domains rather than inventing type constructors.
- `$0` is the whole record; missing fields are empty strings. Ranges such as
  `$3..` preserve separators and are unavailable with `--csv`.
- A single top-level value prints automatically. `(p VALUE ...)` prints values
  separated by spaces; use `s/join` for another delimiter and `csv/join` for CSV
  encoding. Filters alone print the original record when they pass. A false
  filter skips the remaining expressions for that record.
- `--call` supplies `$0` as the first argument; `--no-input --call` (or `-nc`)
  supplies only explicit arguments and runs once. For nested expressions or a
  different primary field, use regular program syntax. `--file` reads that
  same syntax from a UTF-8 file while stdin remains available for input records.
- With `--call`, pass regex patterns without regex-literal `/`
  delimiters, for example `cho -c re/ex 'id=(\w+)' 1`.
- To split a field or transformed value into a local record, consult
  `cho --help s/with` or `cho --help re/with` for literal or regex delimiters.

Small starting points:

```sh
printf 'Alice 18\nBob 30\n' | cho '(f (> $2 20)) (p $1)'
# Bob
printf '2026-08-24T01:30:00Z\n' | cho '(dt/fmt $1 "%Y-%m-%d %H:%M" "Asia/Tokyo")'
# 2026-08-24 10:30
printf '10.1.2.3\n8.8.8.8\n' | cho '(f (cidr/contains? "10.0.0.0/8" $1))'
# 10.1.2.3
cho -nc uuid/v4
# One generated UUID
```

In a cho checkout, `examples/` contains complete pipelines and sample data.
Use relevant examples without treating them as an exhaustive function catalog.

## Handle errors and verify

Test a representative sample before processing a full dataset when executing
an unfamiliar transformation. Check stdout, stderr, and exit status separately.
Include boundaries or malformed values when they matter to the requested
filtering, validation, or recovery behavior; do not invent a full test suite
for a simple command explanation.

A typed conversion failure can indicate malformed data or a wrong column.
Do not silently discard records or switch to string comparisons to suppress it.
`default` recovers from both an empty value and an evaluation error; scope it to
the expression for which a fallback is intended. Boolean false and numeric zero
are not empty values.

Output from earlier records remains if a later record fails. Nonempty stdout
does not prove completion. In Bash pipelines, use `set -o pipefail` when later
commands could hide cho's failure. When executing commands, report the observed
result and any failure; when only drafting them, do not imply they were run.
