#!/bin/sh
set -eu

# ヘッダーを除き、年齢が25以上の人だけを表示する。
cat examples/people.csv |
    cho --csv '(f (s/!= $1 "name")) (f (>= $2 25)) (p $1 $2)'
