;;; cho-mode.el --- Edit shell commands containing Cho programs -*- lexical-binding: t; -*-

(require 'sh-script)
(require 'seq)
(require 'eldoc)
(require 'cho-mode-data)

(declare-function paredit-mode "paredit" (&optional arg))
(declare-function rainbow-delimiters-mode "rainbow-delimiters" (&optional arg))
(declare-function cho-mode-generate-data "generate-cho-mode-data"
                  (&optional metadata-file output-file))

;;;###autoload
(defun cho-mode-update-metadata (&optional metadata-file)
  "Update checked-in mode data from Cho METADATA-FILE.

With no argument, read metadata.json in the repository root.  Starting
`cho-mode' never reads or regenerates the JSON metadata."
  (interactive)
  (require 'generate-cho-mode-data)
  (cho-mode-generate-data metadata-file))

(defconst cho-mode--function-regexp
  (concat "(?[ \t\n]*\\(" (regexp-opt cho-mode--functions)
          "\\)\\(?:[ \t\n)]\\)"))

(defvar-local cho-mode--parent-syntax-propertize-function nil)

(defun cho-mode--shell-word (limit)
  "Read one shell word before LIMIT and return (TEXT START END QUOTED).
QUOTED is non-nil only when the whole word is one single-quoted string."
  (let ((start (point)) (quote-char nil) (escaped nil))
    (while (and (< (point) limit)
                (or quote-char escaped
                    (not (memq (char-after) '(?\s ?\t ?\n ?\; ?\| ?\& ?\< ?>)))))
      (let ((char (char-after)))
        (cond
         (escaped (setq escaped nil))
         ((eq quote-char ?')
          (when (eq char ?')
            (setq quote-char nil)))
         ((eq quote-char ?\")
          (cond ((eq char ?\\) (setq escaped t))
                ((eq char ?\") (setq quote-char nil))))
         ((eq char ?\\) (setq escaped t))
         ((memq char '(?' ?\")) (setq quote-char char)))
        (forward-char)))
    (let ((word (buffer-substring-no-properties start (point))))
      (list word start (point) (string-match-p "\\`'[^']*'\\'" word)))))

(defun cho-mode--program-regions (&optional limit)
  "Return interiors of single-quoted Cho programs before LIMIT.
Only an unquoted `cho' command word can introduce a program."
  (save-excursion
    (goto-char (point-min))
    (let ((limit (or limit (point-max))) (command-start t) cho-command regions)
      (while (< (point) limit)
        (let ((char (char-after)))
          (cond
           ((memq char '(?\s ?\t)) (forward-char))
           ((eq char ?\n)
            (setq command-start t cho-command nil)
            (forward-char))
           ((memq char '(?\; ?\| ?\&))
            (setq command-start t cho-command nil)
            (forward-char))
           ((memq char '(?< ?>)) (forward-char))
           ((eq char ?#)
            (forward-line 1)
            (setq command-start t cho-command nil))
           (t
            (pcase-let ((`(,word ,start ,end ,single-quoted)
                         (cho-mode--shell-word limit)))
              (when (= start end) (forward-char))
              (cond
               ((and command-start (equal word "cho"))
                (setq cho-command t command-start nil))
               (t
                (when (and cho-command single-quoted
                           (string-match-p "\\`'[ \t\n]*(" word))
                  (push (cons (1+ start) (1- end)) regions)
                  (setq cho-command nil))
                (setq command-start nil))))))))
      (nreverse regions))))

(defun cho-mode--in-program-p (position)
  (save-match-data
    (seq-some (lambda (region)
                (and (<= (car region) position) (< position (cdr region))))
              (cho-mode--program-regions))))

(defun cho-mode--syntax-propertize (start end)
  ;; Retain sh-mode's here-doc and substitution handling first.
  (when cho-mode--parent-syntax-propertize-function
    (funcall cho-mode--parent-syntax-propertize-function start end))
  ;; Shell quotes hide parentheses from scan-sexps.  Make only the quotes around
  ;; a Cho program punctuation, so paredit and show-paren can see its S-exprs.
  (dolist (region (cho-mode--program-regions end))
    (let ((open (1- (car region)))
          (close (cdr region)))
      (when (and (< open end) (>= close start))
        (put-text-property open (1+ open) 'syntax-table (string-to-syntax "."))
        (put-text-property close (1+ close) 'syntax-table (string-to-syntax "."))))))

(defun cho-mode--font-lock-matcher (regexp limit)
  "Search REGEXP up to LIMIT, accepting matches inside Cho programs only."
  (let (found)
    (while (and (not found) (re-search-forward regexp limit t))
      (when (cho-mode--in-program-p (match-beginning 0))
        (setq found t)))
    found))

(defun cho-mode--match-function (limit)
  (cho-mode--font-lock-matcher cho-mode--function-regexp limit))

(defun cho-mode--match-field (limit)
  (cho-mode--font-lock-matcher "\\$[0-9]+\\_>\\|\\_<\\(?:NR\\|NF\\)\\_>" limit))

(defun cho-mode--match-constant (limit)
  (cho-mode--font-lock-matcher "\\_<\\(?:true\\|false\\)\\_>" limit))

(defun cho-mode--match-regexp (limit)
  ;; A regexp literal begins an argument.  Requiring whitespace or an opening
  ;; paren before the slash avoids treating the slash in names such as
  ;; `s/upper' and `s/reverse' as the ends of one large regexp.
  (cho-mode--font-lock-matcher
   "[ (\t\n]\\(/\\(?:\\\\.\\|[^/\n]\\)+/\\)" limit))

(defun cho-mode--argument-index (head-end target form-end)
  "Return the zero-based argument at TARGET after HEAD-END.
FORM-END is the closing parenthesis of the current form."
  (save-excursion
    (goto-char head-end)
    (let ((index 0) result)
      (while (and (not result) (< (point) form-end))
        (skip-chars-forward " \t\n" form-end)
        (if (>= (point) target)
            (setq result index)
          (let ((argument-end
                 (condition-case nil
                     (scan-sexps (point) 1)
                   (scan-error form-end))))
            (if (<= target argument-end)
                (setq result index)
              (goto-char argument-end)
              (setq index (1+ index))))))
      result)))

(defun cho-mode--form-context ()
  "Return (FUNCTION ARGUMENT-INDEX) for the Cho form around point."
  (when (cho-mode--in-program-p (point))
    (save-excursion
      (let* ((target (point))
             (open (if (eq (char-after) ?\()
                       (point)
                     (nth 1 (syntax-ppss)))))
        (when open
          (goto-char (1+ open))
          (skip-chars-forward " \t\n")
          (let ((head-beg (point)))
            (skip-chars-forward "^() \t\n")
            (let* ((head-end (point))
                   (name (buffer-substring-no-properties head-beg head-end))
                   (form-end (condition-case nil
                                 (1- (scan-sexps open 1))
                               (scan-error (point-max)))))
              (when (assoc name cho-mode--eldoc-signatures)
                (list name
                      (and (> target head-end)
                           (cho-mode--argument-index
                            head-end target form-end)))))))))))

(defun cho-mode--highlight-signature-argument (signature argument-index)
  "Highlight ARGUMENT-INDEX in a copy of SIGNATURE."
  (let ((result (copy-sequence signature)))
    (when (integerp argument-index)
      (let ((close (string-match ")" result))
            (token-index -1)
            target-beg target-end)
        (when close
          (with-temp-buffer
            (insert (substring result 0 close))
            (goto-char (point-min))
            (while (re-search-forward "[^() \t\n]+" nil t)
              (setq token-index (1+ token-index))
              (when (= token-index (1+ argument-index))
                (setq target-beg (1- (match-beginning 0))
                      target-end (1- (match-end 0)))))
            ;; Variadic calls keep the ellipsis highlighted for later args.
            (unless target-beg
              (goto-char (point-min))
              (when (re-search-forward "\\.\\.\\." nil t)
                (setq target-beg (1- (match-beginning 0))
                      target-end (1- (match-end 0))))))
          (when target-beg
            (add-face-text-property target-beg target-end
                                    'eldoc-highlight-function-argument
                                    nil result)))))
    result))

(defun cho-mode-eldoc-function (&optional _callback)
  "Return Eldoc text for the innermost Cho form at point."
  (pcase (cho-mode--form-context)
    (`(,name ,argument-index)
     (mapconcat
      (lambda (signature)
        (cho-mode--highlight-signature-argument signature argument-index))
      (cdr (assoc name cho-mode--eldoc-signatures))
      "  |  "))))

(defun cho-mode--completion-annotation (candidate)
  "Return a signature annotation for completion CANDIDATE."
  (when-let* ((signature (cadr (assoc candidate cho-mode--eldoc-signatures))))
    (concat "  " signature)))

(defun cho-mode-completion-at-point ()
  "Complete a Cho function at the head of the form around point."
  (when (cho-mode--in-program-p (point))
    (save-excursion
      (let* ((target (point))
             (open (nth 1 (syntax-ppss))))
        (when open
          (goto-char (1+ open))
          (skip-chars-forward " \t\n")
          (let ((beg (point)))
            (skip-chars-forward "^() \t\n")
            (let ((end (point)))
              (when (and (<= beg target) (<= target end))
                (list beg end
                      (mapcar #'car cho-mode--eldoc-signatures)
                      :annotation-function #'cho-mode--completion-annotation
                      :company-kind (lambda (_) 'function)
                      :exclusive 'no)))))))))

(defun cho-mode-indent-or-complete ()
  "Complete a Cho function at point, or indent as in `sh-mode'."
  (interactive)
  (if (cho-mode-completion-at-point)
      (completion-at-point)
    (indent-for-tab-command)))

(defconst cho-mode--font-lock-keywords
  '((cho-mode--match-function 1 font-lock-function-name-face t)
    (cho-mode--match-field 0 font-lock-variable-name-face t)
    (cho-mode--match-constant 0 font-lock-constant-face t)
    (cho-mode--match-regexp 1 font-lock-string-face t)))

;;;###autoload
(define-derived-mode cho-mode sh-mode "Cho"
  "Major mode for shell command lines containing single-quoted Cho programs."
  (setq-local cho-mode--parent-syntax-propertize-function
              syntax-propertize-function)
  (setq-local syntax-propertize-function #'cho-mode--syntax-propertize)
  (font-lock-add-keywords nil cho-mode--font-lock-keywords 'append)
  (syntax-propertize (point-max))
  (add-hook 'eldoc-documentation-functions #'cho-mode-eldoc-function nil t)
  (add-hook 'completion-at-point-functions
            #'cho-mode-completion-at-point nil t)
  (local-set-key (kbd "TAB") #'cho-mode-indent-or-complete)
  (local-set-key (kbd "<tab>") #'cho-mode-indent-or-complete)
  (eldoc-mode 1)
  (when (require 'paredit nil t)
    (funcall #'paredit-mode 1))
  (when (require 'rainbow-delimiters nil t)
    (funcall #'rainbow-delimiters-mode 1)))

(provide 'cho-mode)
;;; cho-mode.el ends here
