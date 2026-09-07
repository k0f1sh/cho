#!/usr/bin/env bash
set -euo pipefail

# Count ERROR records per service; put the highest count first.
# cho selects records and extracts the service; sort and uniq aggregate them.
# C locale makes sorting reproducible. The final cho formats count + service.
# Columns: timestamp, level, service, message (the remaining fields)
export LC_ALL=C
cho '(filter (s/= $2 "ERROR")) (print $3)' <<'LOGS' |
2026-09-01T09:00:00Z INFO api Request completed
2026-09-01T09:00:01Z ERROR worker Queue connection lost
2026-09-01T09:00:02Z ERROR api Database connection timed out
2026-09-01T09:00:03Z WARN api Retrying request
2026-09-01T09:00:04Z ERROR worker Job exceeded retry limit
2026-09-01T09:00:05Z ERROR worker Cannot send notification
LOGS
  sort | uniq -c | sort -k1,1nr -k2,2 |
  cho '(print $2 "|" (str "errors=" $1))'

# Expected output:
# worker | errors=3
# api | errors=1
