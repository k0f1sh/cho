# cho

Filter, extract, and transform text with one-liners that understand numbers,
dates, and IP addresses.

`cho` is an awk-inspired command-line tool with composable Lisp-like expressions.
Compare timestamps, check whether an IP belongs to a network, and format output
without writing parsing code. It bridges the gap between shell one-liners and
small standalone scripts.

```console
# Filter logs by timestamp and CIDR subnet, then format and transform fields:
$ cat access.log | cho '(f (dt/>= $1 "2026-08-01T00:00:00Z")) (f (cidr/contains? "10.0.0.0/8" $2)) (p (dt/fmt $1 "%m-%d %H:%M") (s/join ":" $2 $4))'
```

- **Contextual type coercion**: Fields are strings until a function requires a type. Numbers, timestamps, IPs, and CIDRs convert automatically without manual parsing boilerplate.
- **Batteries included**: Rich built-in primitives for dates, durations, byte sizes, IP/CIDR networking, URLs, SemVer, and regular expressions.
- **First-class CSV & TSV**: Handles quoted fields, embedded newlines, and header skipping (`-s`) out of the box with `--csv` and `--tsv`.
- **Self-documenting CLI**: Built-in search (`cho -k`) and per-function help (`cho --help s/trim`) keep you in the flow without opening a browser.

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

- **Records and fields**: Each line is a record; whitespace splits fields by default (`$0` is the full record; `$1`, `$2`, ... refer to fields). Use `-F`, `--csv`, or `--tsv` to change how fields are parsed.
- **Forms**: `p` (short for `print`) outputs values separated by spaces. `f` (short for `filter`) filters records; filters without an explicit `print` output the whole record.
- **Automatic types**: Fields are strings until a function requires a specific type. In `(> $2 20)`, `$2` is compared as a number and reports an error with line and field info if it cannot be converted.

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

Filter records by human-readable byte sizes without manual conversion:

```console
$ printf '%s\n' \
    'GET /index.html 200 4.2kB' \
    'GET /video.mp4 200 15.4MB' \
    'GET /style.css 200 850B' |
    cho '(f (bs/>= $4 "1MB")) (p $2 $4)'
/video.mp4 15.4MB
```

Chain transformations with the threading macro:

```console
$ echo '  hello-world  ' | cho '(p (-> $0 s/trim (s/replace "-" "_") s/upper))'
HELLO_WORLD
```

Use `s/with` to split one field again and select a field from it. Pass a
delimiter for literal splitting, or omit it to split on whitespace:

```console
$ echo 'job42 ready api:worker:8080' | cho '(p (s/with $3 ":" $2))'
worker
```

When the delimiter needs to be a regular expression, use
`(re/with VALUE PATTERN BODY)` instead.

### Built-in domains

Functions are organized by clear domain prefixes. Below is a selection of representative functions from common domains:

- **Strings & Text**: `s/` (`s/trim`, `s/upper`, `s/replace-all`, `s/join`, `s/with`)
- **Regular Expressions**: `~` (regex match), `re/` (`re/extract`, `re/replace`, `re/with`)
- **Date & Time**: `dt/` (RFC 3339 timestamps: `dt/>=`, `dt/diff`, `dt/fmt`), `d/` (calendar dates: `d/>=`, `d/diff`), `du/` (durations: `du/s`, `du/h`)
- **Networking**: `ip/` (`ip/v4?`, `ip/private?`), `cidr/` (`cidr/contains?`, `cidr/network`, `cidr/prefix`)
- **Web & Identifiers**: `url/` (`url/host`, `url/path`, `url/query`), `semver/` (`semver/>=`), `uuid/` (`uuid/v7`), `ulid/` (`ulid/new`)
- **Numbers & Units**: `n/` (`n/round`, `n/clamp`, `n/fixed`), `bs/` (byte sizes: `bs/to-b`, `bs/>=`), `path/` (`path/name`, `path/ext`)

Run `cho --help` for the complete syntax, functions, and forms, or `cho -k` to search them.

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
