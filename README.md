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

Extract a log message with field ranges — `$3..` keeps everything from the
third field through the end of the record, preserving the original spacing:

```console
$ echo '2026-08-24 INFO service   started successfully' | cho '(p $3..)'
service   started successfully
```

`$2..4` picks an inclusive bounded range — here, fields 2 through 4, preserving the
separators between them:

```console
$ echo 'one  two   three four five' | cho '(p $2..4)'
two   three four
```

Process CSV:

```console
$ printf 'name,city
Alice,"Tokyo, Japan"
Bob,Osaka
' | cho --csv --skip-header '(p (s/join " -> " $1 $2))'
Alice -> Tokyo, Japan
Bob -> Osaka
```

Fields are plain strings, but cho knows about types. When a function
expects a DateTime, IP address, CIDR, URL, or SemVer, the string is converted
automatically — no annotations or casts needed. If the value doesn't match the
expected format, cho stops with an error that pinpoints the record, function,
and argument:

```console
$ printf '%s\n' \
    '2026-08-02T09:00:00Z 10.1.2.3 deploy' \
    '2026-07-31T23:00:00Z 10.2.3.4 old' \
    '2026-08-03T12:00:00Z 8.8.8.8 external' |
    cho '(f (dt/>= $1 "2026-08-01T00:00:00Z")) (f (cidr/contains? "10.0.0.0/8" $2))'
2026-08-02T09:00:00Z 10.1.2.3 deploy
```

`dt/>=` parses `$1` as a DateTime, while `cidr/contains?` parses `$2` as an IP
address. Invalid values fail with a precise error:

```console
$ echo 'not-a-date' | cho '(f (dt/>= $1 "2026-08-01T00:00:00Z"))'
cho: record 1: dt/>=: argument 1 expects DateTime, but "not-a-date" is not valid RFC 3339
```

Chain transformations with the threading macro:

```console
$ echo '  hello-world  ' | cho '(p (-> $1 s/trim (s/replace "-" "_") s/upper))'
HELLO_WORLD
```

cho handles text, numbers, datetime, duration, IP/CIDR, URLs, and semver.
Run `cho --help` for the complete syntax, functions, and special forms.

## Documentation

`cho --help` contains the complete syntax and short command examples.

[`metadata.json`](metadata.json) is the machine-readable index of functions and
forms. Its `schema_version` changes when the JSON structure makes an
incompatible change.

The [`examples`](examples) directory contains scripts and sample data for
reviewing a CSV account export, investigating connection timeouts, checking
release versions, and composing multiple cho processes as a typed pipeline.

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
