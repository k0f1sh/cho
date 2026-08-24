#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# Show timed-out connections from the internal network to private addresses.
cho --tsv --skip-header '
  (filter (s/= $2 "timeout"))
  (filter (cidr/contains? "192.168.0.0/16" $3))
  (filter (ip/private? $4))
  (print (dt/fmt $1 "%H:%M:%S") (s/join " -> " $3 $4))
' < "$example_dir/connection-events.tsv"
