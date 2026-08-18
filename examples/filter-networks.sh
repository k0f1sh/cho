#!/bin/sh
set -eu

# プライベートIPv4アドレスを持つレコードを表示する。
cat examples/people.csv |
    cho --csv '(filter (s/!= $1 "name")) (filter (ip/private? $5)) (print $1 $5)'

# 10.0.0.0/8に含まれるIPアドレスを持つレコードを表示する。
cat examples/people.csv |
    cho --csv '(filter (s/!= $1 "name")) (filter (cidr/contains? "10.0.0.0/8" $5)) (print $1 $5)'
