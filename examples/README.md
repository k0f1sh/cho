# Practical cho examples

Each script answers one concrete question using a small, realistic input.
Open a script to see the column meanings, the cho program, the input records,
and the exact expected output together. All data is fictional.

| Question | Script | What it demonstrates |
| --- | --- | --- |
| Which unpaid invoices are overdue? | [overdue-invoices.sh](overdue-invoices.sh) | CSV quoting, date comparisons, missing owners |
| Which API requests took at least 500 ms? | [analyze-slow-requests.sh](analyze-slow-requests.sh) | URL parsing, numeric thresholds, readable units |
| Which internal connections timed out? | [audit-connection-timeouts.sh](audit-connection-timeouts.sh) | TSV, CIDR membership, private IPs, explicit timezone |
| Which deployments need an update? | [check-release-versions.sh](check-release-versions.sh) | Semantic version ordering, release candidates |
| Which completed jobs exceeded 30 minutes? | [find-long-jobs.sh](find-long-jobs.sh) | Timestamp subtraction across offsets and midnight |
| Which services logged the most errors? | [summarize-errors.sh](summarize-errors.sh) | Filtering with cho, counting with sort and uniq |
| What is the path for the previous day's sales export? | [daily-export-path.sh](daily-export-path.sh) | `-n` to start without stdin, chained `-c` calls, Japan calendar dates |

## Run an example

You need Bash and `cho` on your `PATH`. The error summary also uses `sort` and
`uniq`. From the repository root, run:

```sh
bash examples/overdue-invoices.sh
```

To use the code in this checkout without installing it:

```sh
cargo build
export PATH="$PWD/target/debug:$PATH"
bash examples/overdue-invoices.sh
```

Scripts print only their results to standard output, so you can redirect or
pipe them. They do not write files or contact external services. Review dates
and thresholds are fixed so the documented output stays reproducible.

`daily-export-path.sh` starts from a Unix timestamp argument instead of sample
records. It chains five single-function calls to print a path such as
`2026/08/31/sales.csv`. Run it without arguments for the documented result, or
pass `"$(date +%s)"` to calculate the path for yesterday in Japan time.

## Use your own data

The quoted here-document markers (`<<'CSV'`, `<<'LOGS'`, etc.) keep each input
record on its own line and prevent the shell from expanding its contents.
The TSV example contains literal tabs; the CSV example includes a quoted
customer name containing a comma.

To read an export instead, remove the here-document marker, sample records,
and closing marker, then redirect the file into the same command. For example,
the invoice report becomes:

```sh
cho --csv --skip-header '
  (filter (s/= $5 "unpaid"))
  (filter (d/< $4 "2026-09-01"))
  (print $1 "|" $2 "|" (str $3 " JPY") "|"
    (str "due=" $4) "|" (str "owner=" (default $6 "unassigned")))
' < invoices.csv
```

Match the column order shown in the script and change the review date or
threshold to suit the task. `--skip-header` is for exports with a header row.
The reports use labels and units for reading in a terminal; for CSV output,
compose a row with `csv/join` instead. See `cho --help csv/join` for an example.

Use `cho --help` for the language reference or `cho --help dt/diff` to inspect
one function. Malformed typed values produce an error and a nonzero exit
status; output from earlier records may already have been emitted.

`generate-documentation.rs` is a development utility that regenerates help
and metadata, rather than a text-processing example.
