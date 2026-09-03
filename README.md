# cho

A small, type-aware text processor for the command line.

Inspired by awk, `cho` processes input one record at a time. It fills the gap
between shell one-liners and small standalone scripts with typed values and
composable Lisp-like functions and forms.

`cho` intentionally has no arrays, loops, user-defined functions, variable
bindings, or assignment. It is designed for small, record-oriented
transformations that fit in a readable pipeline.

> [!WARNING]
> `cho` is experimental. Its syntax and behavior may change.

## Install

Rust and Cargo are required.

```console
$ cargo install --git https://github.com/k0f1sh/cho.git
```

To install from a local checkout instead:

```console
$ git clone https://github.com/k0f1sh/cho.git
$ cd cho
$ cargo install --path .
```

## Quick start

Pick fields:

```console
$ echo 'Alice 30 tokyo' | cho '(p $1 $3)'
Alice tokyo
```

Filter and format:

```console
$ printf 'Alice 18
Bob 30
Carol 25
' | cho '(f (> $2 20)) (p $1)'
Bob
Carol
```

By default, cho treats each input line as a record and its whitespace-separated
parts as fields. Use `-F`, `--csv`, or `--tsv` to change how fields are parsed.
`$1`, `$2`, ... refer to those fields. `p` is short for `print`, `f` for
`filter`. Filters without an explicit `print` output the whole record.

## Examples

Keep everything from the third field through the end of the record:

```console
$ echo '2026-08-24 INFO service   started successfully' | cho '(p $3..)'
service   started successfully
```

Filter CSV or TSV data with typed comparisons (dates, sizes, IPs, and more):

```console
$ cho --csv -s '(f (dt/>= $3 "2026-08-01T00:00:00Z")) (p $1 $2)'
```

Filter by timestamp and CIDR block without manual type parsing:

```console
$ printf '%s\n' \
    '2026-08-02T09:00:00Z 10.1.2.3 deploy' \
    '2026-07-31T23:00:00Z 10.2.3.4 old' \
    '2026-08-03T12:00:00Z 8.8.8.8 external' |
    cho '(f (dt/>= $1 "2026-08-01T00:00:00Z")) (f (cidr/contains? "10.0.0.0/8" $2))'
2026-08-02T09:00:00Z 10.1.2.3 deploy
```

Chain transformations with the threading macro:

```console
$ echo '  hello-world  ' | cho '(p (-> $1 s/trim (s/replace "-" "_") s/upper))'
HELLO_WORLD
```

cho handles text, numbers, dates, durations, byte sizes, IPs, URLs, semver, and
more. Run `cho --help` for the complete syntax, functions, and special forms.

## Documentation

`cho --help` contains the complete syntax and short command examples.
Use `cho --help s/trim` to show the signatures, examples, and notes for one
function or form.
Use `cho -k` to list function and form names, or `cho -k trim` to search them
and show matching summaries.
Help, apropos, and version are standalone commands; execution options and
programs are not combined with them.

[`metadata.json`](metadata.json) is the machine-readable index of functions and
forms. Its `schema_version` changes when the JSON structure makes an
incompatible change.

The [`examples`](examples) directory contains scripts and sample data for
reviewing a CSV account export, auditing connection timeouts, checking
release versions, and analyzing slow requests across a typed pipeline.

## Development

```console
$ cargo fmt --check
$ cargo test
$ cargo clippy --all-targets -- -D warnings
```

Regenerate the checked-in help and callable metadata after changing the
language registry:

```console
$ cargo run --quiet --features documentation --example generate-documentation
```

## The name

In Japanese, "awk" sounds a little like *oku* (億, 10^8). *Chō* (兆,
10^12) is the next named large-number unit. It also happens to stand for
Composable, Handles typed values, One-liner friendly.

## License

[MIT](LICENSE)
