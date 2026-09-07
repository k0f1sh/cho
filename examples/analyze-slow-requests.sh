#!/usr/bin/env bash
set -euo pipefail

# Find API requests taking at least 500 ms, including failed requests.
# Compare the parsed hostname so a URL merely mentioning the API is excluded.
# Columns: timestamp, method, url, status, elapsed_ms
cho '
  (filter (s/= (url/host $3) "api.example.com"))
  (filter (>= $5 500))
  (print (dt/fmt $1 "%H:%M:%S UTC") $2 (url/path $3)
    (str "status=" $4) (str "elapsed=" $5 "ms"))
' <<'REQUESTS'
2026-09-01T09:00:00Z GET https://api.example.com/v1/users 200 120
2026-09-01T09:00:01Z POST https://api.example.com/v1/orders?source=web 201 810
2026-09-01T09:00:02Z GET https://web.example.com/help 200 950
2026-09-01T09:00:03Z GET https://api.example.com/v1/reports 504 3000
2026-09-01T09:00:04Z GET https://api.example.com/v1/search?q=book 200 500
REQUESTS

# Expected output:
# 09:00:01 UTC POST /v1/orders status=201 elapsed=810ms
# 09:00:03 UTC GET /v1/reports status=504 elapsed=3000ms
# 09:00:04 UTC GET /v1/search status=200 elapsed=500ms
