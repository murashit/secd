(define caar (lambda (x) (car (car x))))
(define cadr (lambda (x) (car (cdr x))))
(define cdar (lambda (x) (cdr (car x))))
(define cddr (lambda (x) (cdr (cdr x))))

(define caaar (lambda (x) (car (caar x))))
(define caadr (lambda (x) (car (cadr x))))
(define cadar (lambda (x) (car (cdar x))))
(define caddr (lambda (x) (car (cddr x))))
(define cdaar (lambda (x) (cdr (caar x))))
(define cdadr (lambda (x) (cdr (cadr x))))
(define cddar (lambda (x) (cdr (cdar x))))
(define cdddr (lambda (x) (cdr (cddr x))))

(define (zero? x) (eq? x 0))

(define (map fn ls)
  (if (null? ls)
      '()
      (cons (fn (car ls)) (map fn (cdr ls)))))

(define (append xs ys)
  (if (null? xs)
      ys
      (cons (car xs) (append (cdr xs) ys))))

; quasiquoteまわりについては大部分下記を参考にさせていただいた
; http://www.geocities.jp/m_hiroi/func/abcscm36.html#appendix1
(define unquote
  (lambda (x) (error)))

(define unquote-splicing
  (lambda (x) (error)))

(define translator-sub
  (lambda (sym ls n succ)
    (list 'list
          (list 'quote sym)
          (translator ls (+ n succ)))))

(define translator-unquote
  (lambda (ls n)
    (list 'cons
          (if (zero? n)
              (cadar ls)
              (translator-sub 'unquote (cadar ls) n -1))
          (translator (cdr ls) n))))

(define translator-unquote-splicing
  (lambda (ls n)
    (if (zero? n)
        (list 'append (cadar ls) (translator (cdr ls) n))
        (list 'cons
              (translator-sub 'unquote-splicing (cadar ls) n -1)
              (translator (cdr ls) n)))))

(define translator-quasiquote
  (lambda (ls n)
    (list 'cons
          (translator-sub 'quasiquote (cadar ls) n 1)
          (translator (cdr ls) n))))

(define translator-list
  (lambda (ls n)
    (if (eq? (caar ls) 'unquote)
        (translator-unquote ls n)
        (if (eq? (caar ls) 'unquote-splicing)
            (translator-unquote-splicing ls n)
            (if (eq? (caar ls) 'quasiquote)
                (translator-quasiquote ls n)
                (list 'cons
                      (translator (car ls) n)
                      (translator (cdr ls) n)))))))

(define translator-atom
  (lambda (ls n)
    (if (eq? (car ls) 'unquote)
        (if (zero? n)
            (cadr ls)
            (if (= n 1)
                (if (eq? (car (cadr ls)) 'unquote-splicing)
                    (list 'cons (list 'quote 'unquote) (cadr (cadr ls)))
                    (translator-sub 'unquote (cadr ls) n -1))
                (translator-sub 'unquote (cadr ls) n -1)))
        (if (eq? (car ls) 'unquote-splicing)
            (if (zero? n)
                (error)
                (if (= n 1)
                    (if (eq? (car (cadr ls)) 'unquote-splicing)
                        (list 'cons (list 'quote 'unquote-splicing) (cadr (cadr ls)))
                        (translator-sub 'unquote-splicing (cadr ls) n -1))
                    (translator-sub 'unquote-splicing (cadr ls) n -1)))
            (if (eq? (car ls) 'quasiquote)
                (translator-sub 'quasiquote (cadr ls) n 1)
                (list 'cons
                      (list 'quote (car ls))
                      (translator (cdr ls) n)))))))

(define translator
  (lambda (ls n)
    (if (pair? ls)
        (if (pair? (car ls))
            (translator-list ls n)
            (translator-atom ls n))
        (list 'quote ls))))

(define-macro (quasiquote x) (translator x 0))

(define-macro (let args . body)
  `((lambda ,(map car args) ,@body) ,@(map cadr args)))

(define-macro (let* args . body)
  (if (null? (cdr args))
      `(let (,(car args)) ,@body)
      `(let (,(car args)) (let* ,(cdr args) ,@body))))

(define-macro (and . args)
  (if (null? args)
      #t
      (if (null? (cdr args))
          (car args)
          `(if ,(car args) (and ,@(cdr args)) #f))))

(define-macro (or . args)
  (if (null? args)
      #f
      (if (null? (cdr args))
          (car args)
          `(if ,(car args) ,(car args) (or ,@(cdr args))))))

(define-macro (cond . args)
  (if (null? args)
      '(undefined)
      (if (eq? (caar args) 'else)
          `(begin ,@(cdar args))
          (if (null? (cdar args))
              `(if ,(caar args) ,(caar args) (cond ,@(cdr args)))
              `(if ,(caar args)
                   (begin ,@(cdar args))
                   (cond ,@(cdr args)))))))
