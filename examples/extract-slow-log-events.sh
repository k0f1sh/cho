#!/usr/bin/env bash
set -euo pipefail

# Find log events taking at least 500 ms, even when fields move around.
# Treat a missing elapsed value as 0 so it falls below the 500 ms threshold.
# Capture group 1 returns just the digits; >= requests their numeric conversion.
# Missing request IDs use a fallback. These logs use unquoted key=value tokens.
cho '
  (filter (>= (default (re/extract $0 /\belapsed=(\d+)ms\b/ 1) 0) 500))
  (print
    (str "request=" (default (re/extract $0 /\brequest_id=([a-z0-9-]+)\b/ 1) "unknown"))
    (re/extract $0 /\belapsed=(\d+)ms\b/))
' <<'LOGS'
2026-09-01T00:00:00Z INFO request_id=req-101 accepted request
2026-09-01T00:00:01Z INFO completed request_id=req-101 elapsed=810ms
2026-09-01T00:00:02Z INFO elapsed=120ms request_id=req-102 completed
2026-09-01T00:00:03Z WARN upstream timeout elapsed=3000ms request_id=req-103
2026-09-01T00:00:04Z INFO completed elapsed=500ms
2026-09-01T00:00:05Z INFO elapsed=499ms request_id=req-104 completed
LOGS

# Expected output:
# request=req-101 elapsed=810ms
# request=req-103 elapsed=3000ms
# request=unknown elapsed=500ms
