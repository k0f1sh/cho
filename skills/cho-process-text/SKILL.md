---
name: cho-process-text
description: Build and verify cho shell one-liners. Use for cho commands or text pipelines that need typed field comparisons and conversions.
---

# Process text with cho

1. Read `cho --help` from the executable you will use. It contains the complete
   syntax, input options, type rules, and examples. Find functions with
   `cho -k QUERY` and check details with `cho --help FUNCTION`; `cho -k` lists
   all function and form names. Run these discovery commands separately from
   execution options and programs.
2. Inspect representative input for delimiters, headers, quoting, empty fields,
   and column meanings. Choose the input format and compose expressions using
   the documented signatures. Keep sorting and cross-record aggregation in
   other pipeline tools. Single-quote programs to protect field references
   from shell expansion.
3. When executing, test a representative sample and check stdout, stderr, and
   exit status. Earlier output remains if a later record fails; use
   `set -o pipefail` in Bash pipelines that could hide cho's failure. Investigate
   typed conversion errors rather than silently discarding records or changing
   comparison types to suppress them.

If cho is unavailable, say so. Distinguish proposed commands from commands you
actually ran, and report observed results and failures when executing.
