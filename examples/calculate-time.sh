#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# 各日時のUTC時間単位への切り下げ、10分後、基準日時からの経過秒数を表示する。
cat "$example_dir/people.csv" |
    cho --csv --skip-header '(print $1 (dt/floor-h $4) (dt/add $4 (du/m 10)) (dt/diff $4 "2026-08-01T00:00:00Z"))'
