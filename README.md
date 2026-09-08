# cho

A small, type-aware text processor for the command line.

Inspired by awk, `cho` processes input one record at a time. It fills the gap
between shell one-liners and small standalone scripts with typed values and
composable Lisp-like functions and forms.

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
`$0` is the whole record; `$1`, `$2`, ... refer to its fields. `p` is short for `print`, `f` for
`filter`. Filters without an explicit `print` output the whole record.

Fields are strings; functions convert them to the types they require, so `>`
above compares `$2` as a number and reports an error if it cannot be converted.

To call just one function on each input line, use `-c`:

```console
$ echo hello | cho -c s/upper
HELLO
```

## Examples

Keep everything from the third field through the end of the record:

```console
$ echo '2026-08-24 INFO service   started successfully' | cho '(p $3..)'
service   started successfully
```

Filter CSV records by timestamp, using `-s` to skip the header:

```console
$ printf '%s\n' \
    'name,role,created_at' \
    'Alice,admin,2026-08-02T09:00:00Z' \
    'Bob,viewer,2026-07-31T12:00:00Z' |
    cho --csv -s '(f (dt/>= $3 "2026-08-01T00:00:00Z")) (p $1 $2)'
Alice admin
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
$ echo '  hello-world  ' | cho '(p (-> $0 s/trim (s/replace "-" "_") s/upper))'
HELLO_WORLD
```

Treat one value as a temporary local record with `s/with`. Inside its body,
`$0`, `$1`, `$2`, ranges, and `NF` refer to the split value; outside it, they
still refer to the original input record. Pass a delimiter for literal
splitting, or omit it to split on whitespace:

```console
$ echo 'job42 ready api:worker:8080' | cho '(p $1 (s/with $3 ":" (s/join ":" $2 $3)))'
job42 worker:8080
```

When the delimiter needs to be a regular expression, use
`(re/with VALUE PATTERN BODY)` instead.

cho handles text, numbers, dates, durations, byte sizes, IPs, URLs, semver, and
more. Run `cho --help` for the complete syntax, functions, and special forms.

`cho` intentionally has no arrays, loops, user-defined functions, variable
bindings, or assignment. It is designed for small, record-oriented
transformations that fit in a readable pipeline.

## Documentation

`cho --help` contains the complete syntax and short command examples.
Use `cho --help s/trim` to show the signatures, examples, and notes for one
function or form.
Use `cho -k` to list function and form names, or `cho -k trim` to search them
and show matching summaries.
Help, apropos, and version are standalone commands; execution options and
programs are not combined with them.
Use `cho -f program.cho` to reuse a longer program from a UTF-8 file while
keeping standard input available for the records being processed.

[`metadata.json`](metadata.json) is the machine-readable index of functions and
forms. Its `schema_version` changes when the JSON structure makes an
incompatible change.

The [`examples`](examples/README.md) directory contains runnable recipes for
overdue invoices, slow API requests, connection timeouts, deployment updates,
long-running jobs, error counts, and daily export paths. Each script includes
sample input and its expected output.

## Development

```console
$ cargo fmt --check
$ cargo test
$ cargo clippy --all-targets -- -D warnings
```

Regenerate the checked-in help and callable metadata after changing the
language registry:

```console
$ cargo run --quiet --features documentation --bin generate-documentation
```

## The name

In Japanese, "awk" sounds a little like *oku* (億, 10^8). *Chō* (兆,
10^12) is the next named large-number unit. It also happens to stand for
Composable, Handy, One-liner friendly.

## License

[MIT](LICENSE)
