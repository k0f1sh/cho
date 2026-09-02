#!/usr/bin/env bash
set -euo pipefail

# Use each cho process as one stage of a typed text pipeline:
# normalize URLs, select slow requests, then format the result. A stage that
# only calls one function can use -c instead of a parenthesized program.
printf '%s\n' \
    'https://api.example.com/v1/users 220' \
    'https://web.example.com/index.html 450' \
    'https://api.example.com/v1/orders 810' |
    cho '(print (url/host $1) $2 (url/path $1))' |
    cho '(filter (> $2 300))' |
    cho '(print $1 $3 (str $2 "ms"))' |
    cho -c s/upper
