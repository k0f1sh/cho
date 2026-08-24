# cho

A small, type-aware text processor for the command line.

Inspired by awk, `cho` processes input one record at a time. It fills the gap
between shell one-liners and small standalone scripts with typed values and
composable Lisp-like expressions.

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

Filter people older than 20, then print two fields as comma-separated values:

```console
$ printf 'Alice 18 tokyo\nBob 30\nCarol 25 osaka\n' |
    cho '(filter (> $2 20)) (print (s/join "," $1 $2))'
Bob,30
Carol,25
```

`$0` is the complete record and `$1`, `$2`, ... are fields. Whitespace separates
fields by default. `print` and `filter` also have the short forms `p` and `f` for
interactive use.

Field ranges select consecutive fields while preserving the separators that
appeared in the input. `$-3` selects through the third field, `$3-` selects from
the third field to the end, and `$2-4` selects the second through fourth fields:

```console
$ printf '2026-08-24 INFO service   started successfully\n' | cho '(p $3-)'
service   started successfully
```

Field ranges work with whitespace, TSV, and `-F REGEX` input. They are not
available with `--csv`, because CSV fields are decoded rather than raw slices of
the input record.

Run `cho --help` for the complete syntax and more examples.

## Typed values and composition

Expressions compose the same way whether selecting fields, parsing dates,
comparing typed values, or applying other transformations.

For example, this keeps records on or after a timestamp and inside a CIDR:

```console
$ printf '%s\n' \
    '2026-08-02T09:00:00Z 10.1.2.3 deploy' \
    '2026-07-31T23:00:00Z 10.2.3.4 old' \
    '2026-08-03T12:00:00Z 8.8.8.8 external' |
    cho '
      (filter (dt/>= $1 "2026-08-01T00:00:00Z"))
      (filter (cidr/contains? "10.0.0.0/8" $2))
    '
2026-08-02T09:00:00Z 10.1.2.3 deploy
```

The fields do not need explicit type annotations or conversion calls. `dt/>=`
expects RFC 3339 datetimes, and `cidr/contains?` expects a CIDR and an IP address.
`cho` parses each string as the type required by the expression. Invalid input
produces an error instead of silently failing to match.

In CSV mode, quoted commas remain part of one field:

```console
$ printf 'name,place\nAlice,"Tokyo, Japan"\nBob,Osaka\n' |
    cho --csv --skip-header '(print (s/join " -> " $1 $2))'
Alice -> Tokyo, Japan
Bob -> Osaka
```

Typical inputs include logs, command output, and delimited files. Built-in value
types cover datetime arithmetic, network addresses, URLs, and release versions.
For JSON input, use `jq`.

## Language basics

For each input record, `cho` evaluates the expressions from left to right. `$0`
contains the whole record; `$1`, `$2`, ... contain its fields. `NR` is the record
number and `NF` is the field count.

When a `filter` fails, `cho` skips the remaining expressions for that record.
A program containing only filters prints each complete record that passes, so
`(print $0)` is unnecessary. An empty program also passes records through
unchanged. Any explicit `print` disables this implicit output. Value expressions
can be nested anywhere a value is accepted.

```console
$ printf 'alice\nalice:admin\n' |
    cho '(print (s/upper $1) (default (s/part ":" 2 $1) "member"))'
ALICE member
ALICE:ADMIN admin
```

`print` evaluates its arguments and separates them with spaces. Use value
expressions such as `str`, `s/join`, and `n/fixed` for formatting.

## Supported values

| Values | Examples |
| --- | --- |
| Text | regex matching, splitting, joining, case conversion, escaping |
| Numbers | arithmetic, truncation, floor/ceil/round, fixed-point formatting |
| DateTime and Duration | RFC 3339 comparison, differences, arithmetic, time zones |
| IP and CIDR | classification, containment, normalized networks and boundaries |
| URL | components, query parameters, percent encoding and decoding |
| SemVer | precedence comparison and component extraction |
| Boolean | composable predicates, `if`, `and`, `or`, and `not` |

Expressions preserve typed results when nested. For example, `dt/diff` returns
a duration, `du/to-h` converts it to hours, and `n/trunc` discards the fractional
part:

```console
$ printf '2026-08-18T02:30:45Z 2026-08-18T00:00:00Z\n' |
    cho '(print (n/trunc (du/to-h (dt/diff $1 $2))))'
2
```

## Input formats

Whitespace-separated input is the default. Use `--csv` for CSV, `--tsv` for
tab-separated input, or `-F REGEX` for another field separator. `--skip-header`
skips the first logical CSV or TSV record.

```console
$ cho --csv --skip-header '(print $1 $3)'
$ cho --tsv '(filter (~ $2 /^api-/)) (print $1)'
$ cho -F ':' '(print $1 $3)'
```

## Errors and recovery

A failed conversion reports the record, expression, and argument number. Missing
fields evaluate to empty strings. Use `default` to provide a fallback for an
empty or invalid value:

```console
$ printf 'Alice tokyo\nBob\n' |
    cho '(print $1 (default $2 "unknown"))'
Alice tokyo
Bob unknown
```

## Documentation

`cho --help` contains the complete syntax and short command examples.

The [`examples`](examples) directory contains scripts and sample data for
reviewing a CSV account export, investigating connection timeouts, and checking
release versions.

## Development

```console
$ cargo fmt --check
$ cargo test
$ cargo clippy --all-targets -- -D warnings
```

## The name

In Japanese, "awk" sounds a little like *oku* (億, 10^8). *Chō* (兆,
10^12) is the next named large-number unit.

## License

[MIT](LICENSE)
