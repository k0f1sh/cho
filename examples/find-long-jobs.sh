#!/usr/bin/env bash
set -euo pipefail

# Find completed jobs that ran for more than 30 minutes.
# Subtract timestamps with explicit offsets, including a job crossing midnight.
# Columns: job, started_at, finished_at
cho '
  (filter (> (du/to-m (dt/diff $3 $2)) 30))
  (print $1 "|" (str "elapsed=" (du/to-m (dt/diff $3 $2)) "min"))
' <<'JOBS'
nightly-backup 2026-08-31T23:40:00+09:00 2026-09-01T00:25:00+09:00
search-index 2026-09-01T00:00:00Z 2026-09-01T00:12:00Z
warehouse-sync 2026-09-01T09:00:00+09:00 2026-09-01T00:35:00Z
report-export 2026-09-01T01:00:00Z 2026-09-01T01:30:00Z
JOBS

# Expected output:
# nightly-backup | elapsed=45min
# warehouse-sync | elapsed=35min
