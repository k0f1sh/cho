#!/bin/sh
set -eu

# 値式を組み合わせて、名前と都市を1つの文字列にする。
cat examples/people.csv |
    cho --csv '(print (str (s/upper $1) ":" (default $3 "unknown")))'
