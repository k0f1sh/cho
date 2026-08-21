#!/bin/sh
set -eu
example_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)

# Format adult accounts created in August or later as a review report.
# This handles CSV quoting, numeric and datetime comparisons, missing values,
# and datetime formatting in one pipeline.
cho --csv --skip-header '
  (filter (and (>= $2 20) (dt/>= $4 "2026-08-01T00:00:00Z")))
  (print (dt/fmt "%Y-%m-%d" $4) $1 (default $3 "unknown"))
' < "$example_dir/people.csv"
