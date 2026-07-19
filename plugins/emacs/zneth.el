(defvar zneth-font-lock-keywords
  (list
   (cons (regexp-opt '("if" "else" "while" "ret" "fn" "constptr" "printf" "readf" 
                       "signed" "unsigned" "struct" "enum" "unsafe" "module" 
                       "void" "static" "i32" "i64" "i128" "f32" "f64" "str" "i8" "i1" "!include") 'words)
         '(font-lock-keyword-face :weight bold))
   (cons (regexp-opt '("true" "false") 'words)
         '(font-lock-constant-face :weight bold))
   '("\\<[0-9]+\\>" . font-lock-number-face)
   '("\\<\\([a-zA-Z_][a-zA-Z0-9_]*\\)\\s-*(" 1 font-lock-function-name-face)
   '("\\<\\(?:signed\\|unsigned\\)\\s-+\\([a-zA-Z_][a-zA-Z0-9_]*\\)\\>" 1 font-lock-variable-name-face))
  "Syntax highlighting rules for zneth-mode.") 

(defvar zneth-mode-syntax-table
  (let ((st (make-syntax-table)))
    (modify-syntax-entry ?\" "\"" st)
    (modify-syntax-entry ?\\ "\\" st)
    (modify-syntax-entry ?/ ". 124b" st)
    (modify-syntax-entry ?\n "> b" st)
    st)
  "Syntax table for zneth-mode.")
(define-derived-mode zneth-mode prog-mode "Zneth"
  "Major mode for editing Zneth files."
  :syntax-table zneth-mode-syntax-table
  (setq font-lock-defaults '(zneth-font-lock-keywords)))
(add-to-list 'auto-mode-alist '("\\.z\\'" . zneth-mode))
(defvar vdo-font-lock-keywords
  (list
   (cons (regexp-opt '("package") 'words)
         '(font-lock-keyword-face :weight bold))
   (cons (regexp-opt '("true" "false") 'words)
         '(font-lock-constant-face :weight bold))
   '("\\<[0-9]+\\>" . font-lock-number-face))
  "Syntax highlighting rules for vdo-mode.")

(defvar vdo-mode-syntax-table
  (let ((st (make-syntax-table)))
    (modify-syntax-entry ?\" "\"" st)
    (modify-syntax-entry ?\\ "\\" st)
    (modify-syntax-entry ?/ ". 124b" st)
    (modify-syntax-entry ?\n "> b" st)
    st)
  "Syntax table for vdo-mode.")
(define-derived-mode vdo-mode prog-mode "VDO"
  "Major mode for editing VDO files."
  :syntax-table vdo-mode-syntax-table
  (setq font-lock-defaults '(vdo-font-lock-keywords)))
(add-to-list 'auto-mode-alist '("\\.vdo\\'" . vdo-mode))
