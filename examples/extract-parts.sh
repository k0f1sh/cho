#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# sedで接続ログをTSVへ前処理してから、choで型付きの判定と表示を行う。
sed -nE 's/^([^ ]+) .*open-conn-track: timeout opening \(TCP ([0-9.]+):[0-9]+ => ([0-9.]+):[0-9]+\)$/\1\t\2\t\3/p' "$example_dir/tailscaled.log" |
    cho --tsv '(f (cidr/contains? "192.168.0.0/16" $2)) (f (ip/private? $3)) (p (dt/fmt "%H:%M:%S" $1) (s/join " -> " $2 $3))'

# 同じ処理を、s/partで必要な部分だけ取り出してcho内で完結させる。
cho '
  (f (~ $0 /open-conn-track: timeout opening \(TCP [0-9.]+:[0-9]+ => [0-9.]+:[0-9]+\)$/))
  (f (cidr/contains? "192.168.0.0/16" (->> $0 (s/part "TCP " 2) (s/part ":" 1))))
  (f (ip/private? (->> $0 (s/part " => " 2) (s/part ":" 1))))
  (p (dt/fmt "%H:%M:%S" $1)
     (s/join " -> "
       (->> $0 (s/part "TCP " 2) (s/part ":" 1))
       (->> $0 (s/part " => " 2) (s/part ":" 1))))
' < "$example_dir/tailscaled.log"

# IPv6の囲みとportも、同じ値式をネストして取り除ける。
printf '%s\n' '[fd00::1]:443' |
    cho '(p (s/part "]:" 1 (s/part "[" 2 $1)))'

# 存在しない位置はstrictなエラーになり、defaultを使った箇所だけ回復する。
if printf '%s\n' 'not-an-endpoint' | cho '(p (s/part ":" 2 $1))'; then
    echo 'expected cho to fail' >&2
    exit 1
else
    status=$?
    echo "cho exited with status $status" >&2
fi
printf '%s\n' 'not-an-endpoint' |
    cho '(p (default (s/part ":" 2 $1) "missing"))'
