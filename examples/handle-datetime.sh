#!/bin/sh
set -eu

# 2026年8月以降のレコードをUTCの日付に整形し、日時順へ並べる。
cat examples/people.csv |
    cho --csv '(filter (s/!= $1 "name")) (filter (dt/>= $4 "2026-08-01T00:00:00Z")) (print (dt/fmt "%Y-%m-%dT%H:%M:%SZ" $4) $1)' |
    sort -k1,1
