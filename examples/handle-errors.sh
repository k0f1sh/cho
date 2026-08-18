#!/bin/sh
set -eu

# ヘッダーのageはNumberに変換できないため、診断を出して非0で終了する。
if cat examples/people.csv |
    cho --csv '(filter (>= $2 20)) (print $1)'
then
    echo "expected cho to fail" >&2
    exit 1
else
    status=$?
    echo "cho exited with status $status" >&2
fi
