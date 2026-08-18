#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# 値式を組み合わせて、名前と都市を1つの文字列にする。
cat "$example_dir/people.csv" |
    cho --csv '(print (str (s/upper $1) ":" (default $3 "unknown")))'
