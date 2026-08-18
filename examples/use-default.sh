#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# 空の都市だけをunknownへ置き換える。
cat "$example_dir/people.csv" |
    cho --csv '(print $1 (default $3 "unknown"))'

# ヘッダーの数値変換エラーだけをdefaultで明示的に回復する。
cat "$example_dir/people.csv" |
    cho --csv '(print (default (if (>= $2 20) $1 $1) "not-a-record"))'
