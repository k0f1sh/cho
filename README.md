# cho

Small tool, expressive one-liners.\
Built for the AWKward bits of shell scripting.

`people.txt`:

```text
Alice 18 tokyo
Bob 30
Carol 25 osaka
```

```console
$ cat people.txt | cho '(filter (> $2 20)) (print (join "," $1 $2))'
Bob,30
Carol,25
```

Fields are `$1`, `$2`, ...; `$0` is the whole record. Expressions compose, so
filtering, formatting, defaults, case conversion, regex matching, and more can be
nested wherever a value is accepted.

```console
# Pick fields
$ cat people.txt | cho '(print $1 $2)'
Alice 18
Bob 30
Carol 25

# Compose value expressions
$ cat people.txt |
    cho '(print (str (upper $1) ":" (default $3 "unknown")))'
ALICE:tokyo
BOB:unknown
CAROL:osaka
```

`places.csv`:

```csv
Alice,"Tokyo, Japan"
Bob,Osaka
```

```console
# Read real CSV, including quoted commas
$ cat places.csv | cho --csv '(print (join " -> " $1 $2))'
Alice -> Tokyo, Japan
Bob -> Osaka
```

Whitespace-separated input is the default. CSV, TSV, and regular-expression field
separators are also supported.

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
