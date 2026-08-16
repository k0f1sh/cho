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

## Quick start

cho reads text from standard input. For each line, `$1`, `$2`, ... refer to
whitespace-separated fields, and `$0` refers to the whole line.

```console
$ echo 'Alice 20' | cho '(print $1)'
Alice
```

## Examples

### Select and format fields

Print the second field from each line:

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

Join values without separators using `str`:

```console
$ printf 'Alice 20\nBob 30\n' | cho '(print (str $1 ":" $2))'
Alice:20
Bob:30
```

Join values with a separator using `join`:

```console
$ printf 'Alice 20 Tokyo\nBob 30 Osaka\n' | cho '(print (join "," $1 $2 $3))'
Alice,20,Tokyo
Bob,30,Osaka
```

### Work with strings

Count Unicode characters:

```console
$ printf 'Alice\n東京\n' | cho '(print $1 (count $1))'
Alice 5
東京 2
```

Escape backslashes and control characters with `escape`. Newlines, carriage returns, and
tabs become the visible sequences `\\n`, `\\r`, and `\\t`, which keeps each value on one
output line without losing where those characters occurred:

```console
$ printf 'first\tsecond\n' | cho '(print (escape $0))'
first\\tsecond
```

`count` can also be used in a filter:

```lisp
(filter (> (count $1) 3))
(print $1)
```

### Run multiple expressions

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

### Filter lines

Filter by a numeric comparison:

```console
$ printf 'Alice 18\nBob 30\nCarol 25\n' | cho '(filter (> $2 20)) (print $1 $2)'
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
$ printf 'Alice 20\nBob 30\n' | cho '(filter (= $1 "Alice")) (print $0)'
Alice 20
```

Multiple filters work like an AND condition:

```lisp
(filter (> $2 20))
(filter (< $2 40))
(print $1)
```

Combine filters with `not`, `and`, and `or`:

```lisp
(filter (and (> $2 20) (< $2 40)))
(print $1 $2)
```

A failed filter skips the remaining expressions for that input line. Expressions before the
filter have already run.

### Match regular expressions

Filter the complete line with a regular expression:

```console
$ printf 'info: ready\nerror: failed\n' | cho '(filter (reg /^error:/)) (print NR $0)'
2 error: failed
```

Or match a specific value:

```console
$ printf 'Alice 20\nbob 30\n' | cho '(filter (reg $1 /^[A-Z]/)) (print $1)'
Alice
```

`~` is a short form of `reg`. Backslashes in `/.../` regular expression literals are
preserved, so regex escapes need no extra escaping:

```console
$ printf '123\n12a\n456\n' | cho '(filter (~ $1 /^\d+$/)) (print $1)'
123
456
```

Escape a literal `/` as `\/`. String patterns remain supported, but their backslashes
must be escaped: `(reg $1 "^\\d+$")`.

### Change the field separator

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

Use `--tsv` as a short form for a tab field separator:

```console
$ printf 'Alice\t20\tTokyo\nBob\t30\tOsaka\n' | cho --tsv '(print $1 $3)'
Alice Tokyo
Bob Osaka
```

`--tsv`, `--csv`, and `-F` cannot be used together. TSV mode only splits on tabs; it
does not apply CSV-style quoting rules.

### Read CSV

Parse CSV records with `--csv`. Quoted commas, escaped quotes, CRLF input, and newlines
inside quoted fields are supported:

```console
$ printf 'Alice,"Tokyo, Japan"\nBob,Osaka\n' | \
    cho --csv '(print $1 $2)'
Alice Tokyo, Japan
Bob Osaka
```

In CSV mode, `$1`, `$2`, ... are decoded fields, `NF` is the number of CSV fields, and
`NR` counts logical CSV records. `$0` preserves the original logical record without its
terminating newline. `--csv` and `-F` cannot be used together.

CSV fields may contain newlines. Use `escape` when downstream commands or terminal output
need one physical line per CSV record:

```console
$ cho --csv '(print NF (escape $9))' < weather.csv
9 雷を伴う雨\\n沿岸部では強風にも注意
```

## Development

```console
$ cargo test
$ cargo clippy --all-targets -- -D warnings
```

## The name

awk sounds like 「億」 in Japanese, so cho comes next: 「兆」.

## License

[MIT](LICENSE)
