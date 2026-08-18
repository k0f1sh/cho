#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# ヘッダーを除き、年齢が25以上の人だけを表示する。
cat "$example_dir/people.csv" |
    cho --csv --skip-header '(f (>= $2 25)) (p $1 $2)'
