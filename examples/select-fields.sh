#!/bin/sh
set -eu

# CSVの名前と都市を取り出す。引用符付きカンマも1フィールドとして扱われる。
cat examples/people.csv |
    cho --csv '(print $1 (default $3 "unknown"))'
