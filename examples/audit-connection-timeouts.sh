#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# Extract connection timeouts from tailscaled logs and show only connections
# from the internal network to private addresses.
sed -nE 's/^([^ ]+) .*open-conn-track: timeout opening \(TCP ([0-9.]+):[0-9]+ => ([0-9.]+):[0-9]+\)$/\1\t\2\t\3/p' "$example_dir/tailscaled.log" |
    cho --tsv '
      (filter (cidr/contains? "192.168.0.0/16" $2))
      (filter (ip/private? $3))
      (print (dt/fmt "%H:%M:%S" $1) (s/join " -> " $2 $3))
    '
