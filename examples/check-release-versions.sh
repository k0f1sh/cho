#!/usr/bin/env bash
set -euo pipefail

# Find deployments older than the required version 1.10.0.
# SemVer orders 1.9.0 before 1.10.0, and 1.10.0-rc.1 before 1.10.0.
# Remove the deployment label's leading "v" before comparing versions.
# Columns: service, deployed_version
cho '
  (filter (semver/< (s/ltrim $2 "v") "1.10.0"))
  (print $1 "|" (str "deployed=" $2) "|" "minimum=v1.10.0")
' <<'DEPLOYMENTS'
api v1.9.0
worker v1.10.0
scheduler v1.10.0-rc.1
web v1.11.2
DEPLOYMENTS

# Expected output:
# api | deployed=v1.9.0 | minimum=v1.10.0
# scheduler | deployed=v1.10.0-rc.1 | minimum=v1.10.0
