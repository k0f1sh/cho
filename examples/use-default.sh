#!/bin/sh
set -eu

# 空の都市だけをunknownへ置き換える。
cat examples/people.csv |
    cho --csv '(print $1 (default $3 "unknown"))'

# ヘッダーの数値変換エラーだけをdefaultで明示的に回復する。
cat examples/people.csv |
    cho --csv '(print (default (if (>= $2 20) $1 $1) "not-a-record"))'
