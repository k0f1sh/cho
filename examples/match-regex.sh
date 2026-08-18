#!/bin/sh
set -eu

# AまたはBで始まる名前のレコードを表示する。
cat examples/people.csv |
    cho --csv '(filter (~ $1 /^[AB]/)) (print $0)'
