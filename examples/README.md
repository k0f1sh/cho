# Practical cho examples

Each script answers one concrete question using a small, realistic input.
Open a script to see the column meanings, the cho program, the input records,
and the exact expected output together. All data is fictional.

| Question | Script | What it demonstrates |
| --- | --- | --- |
| Which unpaid invoices are overdue? | [overdue-invoices.sh](overdue-invoices.sh) | CSV quoting, date comparisons, missing owners |
| Which API requests took at least 500 ms? | [analyze-slow-requests.sh](analyze-slow-requests.sh) | URL parsing, numeric thresholds, readable units |
| Which free-form log events took at least 500 ms? | [extract-slow-log-events.sh](extract-slow-log-events.sh) | Regex captures, whole-match extraction, numeric comparison, missing IDs |
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
