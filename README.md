```
% cat test.scm
(define (fib n)
  (cond ((eq? n 0) 0)
        ((eq? n 1) 1)
        (else (+ (fib (- n 1)) (fib (- n 2))))))
(print (fib 30))

% time ./target/release/secd test.scm
832040
./target/release/secd test.scm  7.40s user 0.05s system 99% cpu 7.502 total
```
