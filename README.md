# cho

Small tool, expressive one-liners.\
Built for the AWKward bits of shell scripting.

## Quick start

cho reads records from standard input and transforms them with small,
composable expressions.

For example, suppose `people.txt` contains whitespace-separated fields:

```text
Alice 18 tokyo
Bob 30
Carol 25 osaka
```

The following command keeps people older than 20 and prints their names and
ages as comma-separated values:

```console
$ cat people.txt | cho '(f (> $2 20)) (p (s/join "," $1 $2))'
Bob,30
Carol,25
```

`p` and `f` are short forms of `print` and `filter`.

Fields are `$1`, `$2`, ...; `$0` is the whole record. Expressions compose, so
filtering, formatting, defaults, case conversion, regex matching, and more can be
nested wherever a value is accepted.

## Context-aware values

cho can also treat text as a meaningful value when an expression asks for it:

```console
$ cat access.log | cho '(filter (dt/>= $1 "2026-08-01T00:00:00Z")) (filter (cidr/contains? "10.0.0.0/8" $2)) (print $0)'
```

There are no date or IP constructors to wrap around every field. `dt/>=` expects
dates and `cidr/contains?` expects a CIDR and an IP address, so cho converts their
string arguments in context. Invalid input is an error instead of a silent false;
use `default` only where recovery is intentional.

```console
$ cho '(print (dt/floor-m (dt/now)))'
2026-08-18T12:34:00Z
```

Duration units stay explicit and compose with datetime arithmetic:

```console
$ cat people.txt | cho '(filter (= NR 1)) (print (du/h 1))'
3600
```

Numeric arithmetic converts string fields in context and uses binary operators:

```console
$ printf '10 2.5\n' | cho '(print (+ $1 $2) (* $1 2))'
12.5 20
```

`+`, `-`, `*`, and `/` accept exactly two numbers. Invalid numbers, division by
zero, and non-finite results are errors.

## Composing expressions

```console
# Pick fields
$ cat people.txt | cho '(print $1 $2)'
Alice 18
Bob 30
Carol 25

# Compose value expressions
$ cat people.txt |
    cho '(print (str (s/upper $1) ":" (default $3 "unknown")))'
ALICE:tokyo
BOB:unknown
CAROL:osaka
```

Extract one literal-delimited part without introducing an array:

```console
$ printf '192.168.10.20:39652 SRC=10.0.0.25\n' |
    cho '(print (s/part ":" 1 $1) (s/part "=" 2 $2))'
192.168.10.20 10.0.0.25
```

If the requested part does not exist, `s/part` returns an empty string, so it
can be composed directly or handled with `default`:

```console
$ printf 'alice\nalice:admin\n' |
    cho '(print $1 (default (s/part ":" 2 $1) "member"))'
alice member
alice:admin admin
```

## Input formats

For example, suppose `places.csv` contains a quoted comma:

```csv
name,place
Alice,"Tokyo, Japan"
Bob,Osaka
```

```console
# Read real CSV, including quoted commas
$ cat places.csv | cho --csv --skip-header '(print (s/join " -> " $1 $2))'
Alice -> Tokyo, Japan
Bob -> Osaka
```

Whitespace-separated input is the default. CSV, TSV, and regular-expression field
separators are also supported. Use `--skip-header` with CSV or TSV input to skip
its first logical record.

Run `cho --help` for the complete syntax and more examples.

> [!WARNING]
> cho is experimental. Its syntax and behavior may change.

## Install

```console
$ git clone https://github.com/k0f1sh/cho.git
$ cd cho
$ cargo install --path .
```

## Development

```console
$ cargo fmt --check
$ cargo test
$ cargo clippy --all-targets -- -D warnings
```

## The name

awk sounds like 「億」 in Japanese, so cho comes next: 「兆」.

## License

[MIT](LICENSE)
