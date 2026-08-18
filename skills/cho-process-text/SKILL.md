---
name: cho-process-text
description: Use cho to inspect, filter, transform, and format line-oriented text, CSV, TSV, and logs with safe Lisp-like one-liners. Use when an agent should process Unix text streams, compare numbers or RFC 3339 timestamps, calculate time differences, classify IP addresses, test CIDR membership, or replace a longer awk/sed pipeline while detecting malformed typed input.
---

# Process text with cho

Build the smallest composable cho program that satisfies the task. Treat the installed
`cho --help` as the source of truth instead of relying on remembered syntax.

## Workflow

1. Run `cho --help` and confirm the installed version exposes the required expressions and
   type signatures. If `cho` is unavailable, report that before proposing an untested command.
2. Inspect the input read-only. Check its delimiter, header, first few records, empty fields,
   quoting, and the intended meaning of each referenced column.
3. Select `--csv`, `--tsv`, `-F`, or the default whitespace splitting based on the actual file. Add `--skip-header` for CSV or TSV input with a header record.
4. Compose a small program from values and predicates shown in help. Let typed expressions
   provide context; do not invent constructors such as `dt`, `num`, `ip`, or `cidr`.
5. Run the command against a small representative input before processing the full stream.
   Verify stdout, stderr, and the exit status separately.
6. Run the full command only after the sample proves the column numbers, types, and quoting.

When working in the cho repository, consult the single-purpose scripts under `examples/` for
tested patterns. Do not copy all of them into the answer.

## Safety rules

- Quote the whole cho program with single shell quotes. Follow the regex escaping guidance in
  `cho --help`; regex literals and quoted strings have different escaping rules.
- Treat a typed conversion error as evidence of malformed data or a wrong column selection.
  Do not silently discard the record or change the comparison type just to make the command pass.
- Expect output from earlier records to remain when a later record fails. A nonzero exit status
  means the overall transformation did not complete even if stdout is nonempty.
- Use `default` only around the smallest value expression whose failure is intentionally
  recoverable. Do not use it to hide errors across an entire record.
- Use `--skip-header` before applying a numeric, datetime, IP, or CIDR predicate to a CSV or TSV file with a header.
- Keep sorting external. Render a sortable key with cho, then pipe to `sort` when ordering is
  required.

## Verification

Return or record the final command together with the observed exit status. For error-handling
tasks, demonstrate both the strict failure and the narrowly recovered form. For commands using
typed values, include at least one valid boundary case and one malformed value in the sample.
