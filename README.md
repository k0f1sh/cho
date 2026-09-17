# cho

Type-aware, line-oriented text processing for the command line.

`cho` is an awk-inspired command-line tool that lets you combine small Lisp-like
expressions. It understands common data types in plain-text fields, so you can
compare and transform them without manual conversion. It fills the gap between
shell one-liners and small scripts.

![Building a backup retention report with cho](cho-demo.gif)

Build a retention report for large backups. `cho` compares human-readable byte
sizes, calculates expiration times, and composes string transformations in one
expression:

```console
$ printf '%s\n' \
    'daily-backup.tar 2GB 2026-09-15T23:30:00+09:00' \
    'video-archive.mp4 850MB 2026-09-16T08:00:00Z' \
    'database-snapshot.sql 8GB 2026-09-10T02:15:00Z' \
    'release-bundle.tar 4GB 2026-09-17T18:45:00-04:00' |
    cho '(f (bs/>= $2 "1GB"))
          (p (s/join " | "
            (-> $1 (s/replace-all "-" "_") s/upper) $2
            (dt/fmt (dt/add $3 (du/d 7)) "%Y-%m-%d %H:%M UTC")))'
DAILY_BACKUP.TAR | 2GB | 2026-09-22 14:30 UTC
DATABASE_SNAPSHOT.SQL | 8GB | 2026-09-17 02:15 UTC
RELEASE_BUNDLE.TAR | 4GB | 2026-09-24 22:45 UTC
```

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

- **Records and fields**: By default, each line is a record and whitespace splits fields (`$0` is the full record; `$1`, `$2`, ... refer to fields). Use `-F`, `--csv`, or `--tsv` to change how input is parsed.
- **Forms**: `p` (short for `print`) outputs values separated by spaces. `f` (short for `filter`) filters records; filters without an explicit `print` output the whole record.
- **Automatic types**: Fields are strings until a function requires a specific type. In `(> $2 20)`, `$2` is compared as a number. If conversion fails, the error identifies the record, function, argument position, and expected type.

`--csv` supports quoted fields and embedded newlines; `--tsv` splits fields
on tabs. Both support header skipping with `-s`.

To call just one function on each input line, use `-c`:

```console
$ echo hello | cho -c s/upper
HELLO
```

## Examples

Filter logs by timestamp and subnet, then format the matching records:

```console
$ printf '%s\n' \
    '2026-08-02T09:00:00Z 10.1.2.3 GET /index.html' \
    '2026-07-31T23:00:00Z 10.2.3.4 GET /old' \
    '2026-08-03T12:00:00Z 8.8.8.8 GET /external' |
    cho '(f (dt/>= $1 "2026-08-01T00:00:00Z"))
         (f (cidr/contains? "10.0.0.0/8" $2))
         (p (dt/fmt $1 "%m-%d %H:%M") (s/join ":" $2 $4))'
08-02 09:00 10.1.2.3:/index.html
```

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

Use `s/with` to split one field again and evaluate an expression using the
resulting fields. Pass a delimiter for literal splitting, or omit it to split
on whitespace:

```console
$ echo 'job42 ready api:worker:8080' | cho '(p (s/with $3 ":" (s/upper $2)))'
WORKER
```

When the delimiter needs to be a regular expression, use
`(re/with VALUE PATTERN BODY)` instead.

You can also calculate elapsed time, extract URL components, and compare
version numbers. Run `cho --help` for the complete syntax and examples,
or `cho -k QUERY` to find a function for your task.

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
