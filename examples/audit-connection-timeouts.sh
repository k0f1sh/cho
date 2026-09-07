#!/usr/bin/env bash
set -euo pipefail

# Investigate timeouts from the application subnet to private destinations.
# The input below uses literal tabs, as a TSV export does.
# Display timestamps in Japan time explicitly, independent of the machine.
cho --tsv --skip-header '
  (filter (s/= $2 "timeout"))
  (filter (cidr/contains? "192.168.10.0/24" $3))
  (filter (ip/private? $4))
  (print (dt/fmt $1 "%Y-%m-%d %H:%M:%S %z" "Asia/Tokyo")
    "|" (s/join " -> " $3 $4) "|" (str "service=" $5))
' <<'TSV'
timestamp	status	source	destination	service
2026-09-01T00:30:00Z	timeout	192.168.10.20	10.0.0.25	postgres
2026-09-01T00:31:00Z	connected	192.168.10.21	10.0.0.25	postgres
2026-09-01T00:32:00Z	timeout	192.168.20.10	10.0.0.26	redis
2026-09-01T00:33:00Z	timeout	192.168.10.22	1.1.1.1	dns
2026-09-01T00:34:00Z	timeout	192.168.10.23	10.0.0.26	redis
TSV

# Expected output:
# 2026-09-01 09:30:00 +0900 | 192.168.10.20 -> 10.0.0.25 | service=postgres
# 2026-09-01 09:34:00 +0900 | 192.168.10.23 -> 10.0.0.26 | service=redis
