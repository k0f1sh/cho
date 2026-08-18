#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# プライベートIPv4アドレスを持つレコードを表示する。
cat "$example_dir/people.csv" |
    cho --csv '(filter (!= NR 1)) (filter (ip/private? $5)) (print $1 $5)'

# 10.0.0.0/8に含まれるIPアドレスを持つレコードを表示する。
cat "$example_dir/people.csv" |
    cho --csv '(filter (!= NR 1)) (filter (cidr/contains? "10.0.0.0/8" $5)) (print $1 $5)'
