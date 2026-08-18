#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# CSVの名前と都市を取り出す。引用符付きカンマも1フィールドとして扱われる。
cat "$example_dir/people.csv" |
    cho --csv '(print $1 (default $3 "unknown"))'
