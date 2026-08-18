#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# 2026年8月以降のレコードをUTCの日付に整形し、日時順へ並べる。
cat "$example_dir/people.csv" |
    cho --csv '(filter (!= NR 1)) (filter (dt/>= $4 "2026-08-01T00:00:00Z")) (print (dt/fmt "%Y-%m-%dT%H:%M:%SZ" $4) $1)' |
    sort -k1,1
