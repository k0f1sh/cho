# cho

cho is a tiny command-line tool for filtering and formatting text with Lisp-like expressions.
It is inspired by awk.

> [!WARNING]
> cho is experimental. Its syntax and behavior may change.

## Install

```console
$ git clone https://github.com/k0f1sh/cho.git
$ cd cho
$ cargo install --path .
```

```console
$ printf 'Alice 20\nBob 30\n' | cho '(print $1)'
Alice
Bob
```

## Examples

Print a field:

```console
$ printf 'Alice 20\nBob 30\n' | cho '(print $2)'
20
30
```

`$0` is the complete input line:

```console
$ printf 'Alice 20\nBob 30\n' | cho '(print $0)'
Alice 20
Bob 30
```

Print several values separated by spaces:

```console
$ printf 'Alice 20\nBob 30\n' | cho '(print $1 "score:" $2)'
Alice score: 20
Bob score: 30
```

Join values without separators using `fmt`:

```console
$ printf 'Alice 20\nBob 30\n' | cho '(print (fmt $1 ":" $2))'
Alice:20
Bob:30
```

Run multiple expressions for every input line:

```console
$ printf 'Alice 20\nBob 30\n' | cho $'(print $1)\n(print $2)'
Alice
20
Bob
30
```

Use `NR` for the line number and `NF` for the number of fields:

```console
$ printf 'Alice 20\nBob 30 Osaka\n' | cho '(print NR NF $1)'
1 2 Alice
2 3 Bob
```

Filter by a numeric comparison:

```console
$ printf 'Alice 18\nBob 30\nCarol 25\n' | \
    cho $'(filter (> $2 20))\n(print $1 $2)'
Bob 30
Carol 25
```

The comparison operators are `>`, `>=`, `<`, `<=`, `=`, and `!=`:

```lisp
(filter (>= $2 20))
(filter (< $2 40))
(print $1 $2)
```

`=` and `!=` also compare strings:

```console
$ printf 'Alice 20\nBob 30\n' | \
    cho $'(filter (= $1 "Alice"))\n(print $0)'
Alice 20
```

Filter the complete line with a regular expression:

```console
$ printf 'info: ready\nerror: failed\n' | \
    cho $'(filter (reg "^error:"))\n(print NR $0)'
2 error: failed
```

Or match a specific value:

```console
$ printf 'Alice 20\nbob 30\n' | \
    cho $'(filter (reg $1 "^[A-Z]"))\n(print $1)'
Alice
```

Choose a field separator with `-F`. The separator is a regular expression:

```console
$ printf 'Alice,20,Tokyo\nBob,30,Osaka\n' | \
    cho -F, '(print $1 $3)'
Alice Tokyo
Bob Osaka
```

`NF` uses the selected separator, while `$0` remains the complete input line:

```console
$ printf 'Alice,20,Tokyo\nBob,30\n' | \
    cho -F ',' '(print NF $1 $0)'
3 Alice Alice,20,Tokyo
2 Bob Bob,30
```

Regular expression separators are supported:

```console
$ printf 'Alice,20;Tokyo\nBob;30,Osaka\n' | \
    cho -F '[,;]' '(print $1 $3)'
Alice Tokyo
Bob Osaka
```

Multiple filters work like an AND condition:

```lisp
(filter (> $2 20))
(filter (< $2 40))
(print $1)
```

A failed filter skips the remaining expressions for that input line. Expressions before the
filter have already run.

## Development

```console
$ cargo test
$ cargo clippy --all-targets -- -D warnings
```

## The name

awk sounds like 「億」 in Japanese, so cho comes next: 「兆」.

## License

[MIT](LICENSE)
