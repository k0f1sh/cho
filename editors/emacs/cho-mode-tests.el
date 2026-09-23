;;; cho-mode-tests.el --- Tests for cho-mode -*- lexical-binding: t; -*-

(require 'ert)
(require 'cho-mode)
(require 'generate-cho-mode-data)

(defun cho-mode-tests--regions (source)
  (with-temp-buffer
    (insert source)
    (cho-mode)
    (mapcar (lambda (region)
              (buffer-substring-no-properties (car region) (cdr region)))
            (cho-mode--program-regions))))

(ert-deftest cho-mode-finds-separate-commands ()
  (should (equal (cho-mode-tests--regions
                  "printf '%s' x | cho -F '[ ]+' '(p $1)'; cho '(f (> $2 3))'\ncho '(p $3)'\n")
                 '("(p $1)" "(f (> $2 3))" "(p $3)"))))

(ert-deftest cho-mode-ignores-quoted-and-non-command-cho ()
  (should (equal (cho-mode-tests--regions
                  "echo \"cho '(p $9)'\"; echo 'cho (p $8)'\n# cho '(p $7)'\nprintf '%s' cho '(p $6)'; cho '(p $1)'\n")
                 '("(p $1)"))))

(ert-deftest cho-mode-ignores-non-program-quotes ()
  (should (equal (cho-mode-tests--regions
                  "cho -F '[ ]+' 'literal' '(p $1)' && cho '(p $2)'\n")
                 '("(p $1)" "(p $2)"))))

(ert-deftest cho-mode-completes-inside-program-only ()
  (with-temp-buffer
    (insert "echo '(p $9)'; cho '(s/up $1)'")
    (cho-mode)
    (goto-char (point-min))
    (search-forward "(p")
    (should-not (cho-mode-completion-at-point))
    (search-forward "(s/up")
    (should (cho-mode-completion-at-point))))

(ert-deftest cho-mode-exposes-program-parentheses ()
  (with-temp-buffer
    (insert "cho '(print (s/trim $1))'")
    (cho-mode)
    (goto-char (point-min))
    (search-forward "'(print")
    (backward-char 6)
    (should (equal (buffer-substring-no-properties
                    (point) (scan-sexps (point) 1))
                   "(print (s/trim $1))"))))

(ert-deftest cho-mode-does-not-highlight-cho-string-contents ()
  (with-temp-buffer
    (insert "cho '(p \"(print $1 true)\" (print $1))'")
    (cho-mode)
    (font-lock-ensure)
    (goto-char (point-min))
    (search-forward "print")
    (should-not (eq (get-text-property (1- (point)) 'face)
                    'font-lock-function-name-face))
    (search-forward "$1")
    (should-not (eq (get-text-property (1- (point)) 'face)
                    'font-lock-variable-name-face))
    (search-forward "true")
    (should-not (eq (get-text-property (1- (point)) 'face)
                    'font-lock-constant-face))
    (search-forward "print")
    (should (eq (get-text-property (1- (point)) 'face)
                'font-lock-function-name-face))))

(ert-deftest cho-mode-generated-data-is-current ()
  (let ((temporary-file (make-temp-file "cho-mode-data-")))
    (unwind-protect
        (progn
          (cho-mode-generate-data nil temporary-file)
          (should (equal (with-temp-buffer
                           (insert-file-contents temporary-file)
                           (buffer-string))
                         (with-temp-buffer
                           (insert-file-contents
                            (expand-file-name "cho-mode-data.el"
                                              cho-mode-generator--directory))
                           (buffer-string)))))
      (delete-file temporary-file))))

;;; cho-mode-tests.el ends here
