#!/usr/bin/env bash
set -euo pipefail

# Count connections by destination /24 network and output network,count CSV.
# Columns: service, destination URL. Each row represents one connection.
# URLs contain IPv4 addresses directly.
# -C selects the URL with @2, places @0 between the separator and prefix length,
# and finally reorders uniq's count + network fields with @2 @1.
# Each cho stage calls one function without an S-expression:
#   URL -> IP -> CIDR -> network address -> network CIDR -> counts -> CSV
# C locale makes sorting reproducible.
export LC_ALL=C
cat <<'CONNECTIONS' |
api http://10.20.4.12:8080/health
worker http://10.20.4.80:9000/jobs
api http://10.20.5.10:8080/health
web http://10.20.6.20:8080/assets/app.js
api http://10.20.4.13:8080/users/42
scheduler http://10.20.5.30:9000/tasks
monitor http://10.20.7.10:9090/metrics
worker http://10.20.4.80:9000/jobs/101
web http://10.20.6.21:8080/assets/styles.css
api http://10.20.5.11:8080/orders/305
api http://10.20.4.12:8080/users/43
worker http://10.20.5.40:9000/jobs/102
web http://10.20.6.20:8080/images/logo.svg
monitor http://10.20.7.11:9090/metrics
scheduler http://10.20.4.90:9000/tasks/daily
api http://10.20.5.10:8080/orders/306
web http://10.20.6.22:8080/assets/app.js
worker http://10.20.4.81:9000/jobs/103
scheduler http://10.20.5.30:9000/tasks/hourly
monitor http://10.20.7.10:9090/health
api http://10.20.4.13:8080/users/44
web http://10.20.6.21:8080/images/banner.png
worker http://10.20.5.40:9000/jobs/104
api http://10.20.4.12:8080/health
CONNECTIONS
  cho -C url/host @2 |
  cho -C s/join / @0 24 |
  cho -C cidr/network @0 |
  cho -C str @0 /24 |
  sort | uniq -c |
  cho -C csv/join @2 @1

# Expected output:
# 10.20.4.0/24,9
# 10.20.5.0/24,7
# 10.20.6.0/24,5
# 10.20.7.0/24,3
