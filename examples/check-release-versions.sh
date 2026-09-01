#!/bin/sh
set -eu

# Select releases at or above the required stable version using SemVer order.
# A string comparison cannot order 1.10.0 and 1.9.0 correctly.
printf '%s\n' \
    'api v1.9.0' \
    'worker v1.10.0' \
    'web v2.0.0-alpha' |
    cho '
      (filter (semver/>= (s/ltrim $2 "v") "1.10.0"))
      (print (s/join "@" $1 $2))
    '
