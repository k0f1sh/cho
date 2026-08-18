#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# AまたはBで始まる名前のレコードを表示する。
cat "$example_dir/people.csv" |
    cho --csv '(filter (~ $1 /^[AB]/)) (print $0)'
