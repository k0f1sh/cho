---
name: cho-process-text
description: Use cho when a shell one-liner or script needs to handle typed data — RFC 3339 datetimes, IP addresses, CIDR ranges, URLs, SemVer — embedded in line-oriented text. Basic shell commands (grep, cut, awk, sed) treat these as opaque strings; cho parses them in context and rejects malformed input instead of silently passing it through. Reach for this skill when the task involves comparing timestamps, testing CIDR membership, extracting URL components, or filtering by version range inside a Unix pipeline.
---

# Process typed data in shell pipelines with cho

cho is a small line-oriented tool that slots into Unix pipelines. Use it when
the data in a field carries meaning that plain string comparison gets wrong:
date ordering, IP classification, CIDR membership, URL parsing, or SemVer
precedence. cho converts fields to the right type in context and treats
invalid input as an error, not a silent mismatch.

If the task only needs pattern matching, field extraction, or string
manipulation that grep / cut / awk / sed already handle well, prefer those
tools. cho adds value when you need type-aware predicates or conversions that
would otherwise require a heavier language.

## When to use cho

- Filtering log lines by an RFC 3339 timestamp range (`dt/>=`, `dt/<`).
- Testing whether an IP address is private, loopback, or inside a CIDR block
  (`ip/private?`, `ip/loopback?`, `cidr/contains?`).
- Extracting or encoding URL components (`url/host`, `url/path`, `url/encode`).
- Comparing version strings by SemVer precedence (`semver/>=`, `semver/<`).
- Arithmetic or fixed-point formatting on numeric fields (`+`, `-`, `n/fixed`).
- Combining the above with field selection, string join, regex match, and
  defaults — all in a single composable expression.

## Workflow

1. Run `cho --help` and confirm the installed version exposes the required
   expressions. If `cho` is unavailable, report that before proposing a command.
2. Inspect the input: delimiter, header, first few records, empty fields,
   quoting, and the meaning of each column you will reference.
3. Select `--csv`, `--tsv`, `-F`, or the default whitespace splitting based on
   the actual file. Add `--skip-header` for CSV or TSV input with a header.
4. Compose a small program from values and predicates shown in help. Typed
   expressions (`dt/>=`, `cidr/contains?`, etc.) convert string arguments in
   context — do not invent constructors such as `dt`, `num`, `ip`, or `cidr`.
5. Run the command against a small representative sample before the full stream.
   Verify stdout, stderr, and the exit status separately.
6. Run the full command only after the sample proves the column numbers, types,
   and quoting are correct.

When working in the cho repository, consult the scripts under `examples/` for
tested patterns. Do not copy all of them into the answer.

## Safety rules

- Quote the whole cho program with single shell quotes. Follow the regex
  escaping guidance in `cho --help`; regex literals and quoted strings have
  different escaping rules.
- Treat a typed conversion error as evidence of malformed data or a wrong
  column selection. Do not silently discard the record or change the comparison
  type just to make the command pass.
- Expect output from earlier records to remain when a later record fails. A
  nonzero exit status means the overall transformation did not complete even if
  stdout is nonempty.
- Use `default` only around the smallest value expression whose failure is
  intentionally recoverable. Do not use it to hide errors across an entire
  record.
- Use `--skip-header` before applying a typed predicate to CSV or TSV with a
  header.
- Keep sorting external. Render a sortable key with cho, then pipe to `sort`
  when ordering is required.

## Verification

Return or record the final command together with the observed exit status. For
error-handling tasks, demonstrate both the strict failure and the narrowly
recovered form. For commands using typed values, include at least one valid
boundary case and one malformed value in the sample.
