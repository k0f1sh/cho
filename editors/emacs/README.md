# Cho mode for Emacs

`cho-mode` extends Emacs `sh-mode` for shell command lines containing a Cho
program in a single-quoted argument, such as:

```sh
printf '%s\n' ' Ada ' | cho '(print (s/trim $1))'
```

It highlights Cho functions, fields, constants, and regular expressions; adds
function-name completion at the start of a form; shows signatures with Eldoc;
and exposes program parentheses to Emacs parenthesis editing. It also loads
`paredit` and `rainbow-delimiters` when those optional packages are installed.
The mode does not execute the command being edited.

## Install and use

Add this directory to Emacs's `load-path`, then enable the mode in a shell
command buffer:

```elisp
(add-to-list 'load-path "/path/to/cho/editors/emacs")
(require 'cho-mode)
;; Run M-x cho-mode in a shell command buffer.
```

For files that you specifically use to edit shell commands, you can register
their path or suffix in `auto-mode-alist`:

```elisp
(add-to-list 'auto-mode-alist '("\\.cho-sh\\'" . cho-mode))
```

The mode recognizes an unquoted `cho` command word at the start of a command,
including after a pipe, `;`, `&&`, or `||`. It finds the first single-quoted
argument whose first non-space character is `(`. It ignores `cho` inside shell
quotes and comments. This covers ordinary one-line commands; complex shell
constructs such as command substitutions and here-documents are not parsed as
full Bash syntax.

## Updating generated data

`cho-mode-data.el` is generated from the repository's `metadata.json`. Run
these commands from the repository root after changing Cho's callable metadata
or version:

```sh
cargo run --quiet --features documentation --bin generate-documentation
emacs -Q --batch -L editors/emacs -l generate-cho-mode-data -f cho-mode-generate-data
emacs -Q --batch -L editors/emacs -l cho-mode-tests.el -f ert-run-tests-batch-and-exit
```

The last command compares regenerated data with the checked-in file, so it
fails when signatures or the Cho version are stale. Normal mode startup only
loads the generated Emacs Lisp file; it does not need `metadata.json`.
