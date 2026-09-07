#!/usr/bin/env bash
set -euo pipefail

# Build the relative path for the previous day's sales export in Japan time.
# An optional argument is the job's execution time in Unix seconds.
# The default is 2026-09-01 09:00 JST; a fixed time keeps this example repeatable.
# Usage: bash examples/daily-export-path.sh [unix_seconds]
# For today's run: bash examples/daily-export-path.sh "$(date +%s)"
# This prints a path; it does not create or read the export file.
#
# -n runs once without reading stdin. With -n -c, all function arguments
# come from the command line. Later -c stages use the entire input line as
# the first argument, followed by any arguments written after the function.
# Each stage performs one operation:
#   Unix seconds -> timestamp -> Japan date -> previous date -> path -> filename
cho -n -c dt/unix "${1:-1788220800}" |
  cho -c dt/fmt '%Y-%m-%d' Asia/Tokyo |
  cho -c d/sub 1 |
  cho -c s/replace-all - / |
  cho -c str /sales.csv

# Expected output (with no arguments):
# 2026/08/31/sales.csv
