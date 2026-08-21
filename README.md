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

IP expressions classify both address families where applicable. `ip/private?`
recognizes RFC 1918 IPv4 addresses and IPv6 unique-local addresses in `fc00::/7`;
`ip/version` returns `4` or `6`:

```console
$ printf '10.0.0.1\nfc00::1\n8.8.8.8\n' |
    cho '(filter (ip/private? $1)) (print $1 (ip/version $1))'
10.0.0.1 4
fc00::1 6
```

CIDR expressions expose the normalized network, prefix length, and range boundaries.
Returned addresses remain typed and can be passed directly to IP expressions:

```console
$ printf '192.168.1.42/24\n2001:db8::1/126\n' |
    cho '(print (cidr/network $1) (cidr/prefix $1) (cidr/last $1) (ip/version (cidr/first $1)))'
192.168.1.0 24 192.168.1.255 4
2001:db8:: 126 2001:db8::3 6
```

`cidr/size` returns the total address count only when it fits Number's safe
integer range. Larger IPv6 ranges are runtime errors and can be handled with
`default`.

URL component expressions parse absolute URLs in context:

```console
$ printf 'https://example.com:8443/a%20b?q=hello%20world#top\n' |
    cho '(print (url/scheme $1) (url/host $1) (url/port $1) (url/path $1) (url/query $1) (url/fragment $1))'
https example.com 8443 /a%20b q=hello%20world top
```

Extracted components keep their percent encoding. A missing optional component
is an empty string, while an invalid URL is an error.

Query parameters can be decoded separately without changing raw `url/query`.
Keys and values use form semantics (`+` is a space), and duplicate keys return
their first value:

```console
$ printf 'https://example.com/?lang=ja&q=hello+world\n' |
    cho '(print (url/query-get "lang" $1) (url/query-get "q" $1) (url/query-has? "page" $1))'
ja hello world false
```

Encode and decode individual URL components explicitly:

```console
$ printf 'hello world\n東京\n' |
    cho '(print (url/encode $0))'
hello%20world
%E6%9D%B1%E4%BA%AC
```

`url/decode` decodes `%XX` escapes but leaves `+` unchanged. Invalid escapes and
decoded bytes that are not UTF-8 are errors.

```console
$ cho '(print (dt/floor-m (dt/now)))'
2026-08-18T12:34:00Z
```

Duration units stay explicit and compose with datetime arithmetic. `du/ms` creates
milliseconds, while `du/d` uses fixed 24-hour days:

```console
$ cat people.txt | cho '(filter (= NR 1)) (print (du/ms 250) (du/d 1))'
0.25 86400
```

Semantic versions compare by SemVer precedence rather than as strings:

```console
$ printf '1.9.0\n1.10.0\n2.0.0-alpha\n' |
    cho '(filter (semver/>= $1 "1.10.0")) (print $1)'
1.10.0
2.0.0-alpha
```

`semver/` comparisons require `MAJOR.MINOR.PATCH`. Build metadata is ignored for
precedence equality; use `s/=` when the complete text must match.

Components are regular values and build metadata remains accepted:

```console
$ printf '1.2.3-alpha.1+build.9\n' |
    cho '(print (semver/major $1) (semver/minor $1) (semver/patch $1) (semver/prerelease $1))'
1 2 3 alpha.1
```

Comparisons and predicates return regular Boolean values. They can be printed,
combined, or selected by `if`, and `filter` accepts any Boolean expression:

```console
$ printf 'prod 10.0.0.1\ndev 127.0.0.1\ndev 8.8.8.8\n' |
    cho '(filter (if (s/= $1 "prod") (ip/private? $2) (ip/loopback? $2))) (print $0)'
prod 10.0.0.1
dev 127.0.0.1
```

The literals are `true` and `false`. Boolean values render with those spellings
and are not implicitly converted from or to strings and numbers. `filter`, `if`,
`not`, `and`, and `or` require Boolean arguments; `and` and `or` short-circuit.

Numeric arithmetic converts string fields in context and uses binary operators:

```console
$ printf '10 2.5\n' | cho '(print (+ $1 $2) (* $1 2))'
12.5 20
```

`+`, `-`, `*`, and `/` accept exactly two numbers. Invalid numbers, division by
zero, and non-finite results are errors.

Use `n/fixed` when output needs a fixed number of digits after the decimal point:

```console
$ printf '3 3.14159\n' | cho '(print (n/fixed 2 $1) (n/fixed 3 $2))'
3.00 3.142
```

`n/fixed` returns a String and accepts a whole digit count from 0 to 100, keeping
calculation separate from final display formatting.

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
