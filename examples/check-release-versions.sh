#!/bin/sh
set -eu

# Select releases at or above the required stable version using SemVer order.
# A string comparison cannot order 1.10.0 and 1.9.0 correctly.
printf '%s\n' \
    'api 1.9.0' \
    'worker 1.10.0' \
    'web 2.0.0-alpha' |
    cho '
      (filter (semver/>= $2 "1.10.0"))
      (print (s/join "@" $1 $2))
    '
