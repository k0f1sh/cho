#!/bin/sh
set -eu

# ヘッダーを除き、年齢が25以上の人だけを表示する。
cat examples/people.csv |
    cho --csv '(filter (s/!= $1 "name")) (filter (>= $2 25)) (print $1 $2)'
