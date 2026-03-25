pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(EXPR_SIZE);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/expr_size.rs)\n";
const EXPR_SIZE: &str = r#";; Compute the tree size of program, not dag size
(function expr_size (Expr) i64 :merge (min old new) )
(function list_expr_size (ListExpr) i64 :merge (min old new))

(rule ((= expr (Function name tyin tyout out)) 
       (= sum (expr_size out))) 
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((= expr (Const n ty assum))) 
      ((set (expr_size expr) 1))  :ruleset always-run)

(rule ((= expr (Top op x y z))
       (= sum (+ (expr_size z) (+ (expr_size y) (expr_size x)))))
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((= expr (Bop op x y)) 
       (= sum (+ (expr_size y) (expr_size x)))) 
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((= expr (Uop op x)) 
       (= sum (expr_size x))) 
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((= expr (Get tup i)) 
       (= sum (expr_size tup))) 
      ((set (expr_size expr) sum)) :ruleset always-run)

(rule ((= expr (Concat x y)) 
       (= sum (+ (expr_size y) (expr_size x)))) 
      ((set (expr_size expr) sum)) :ruleset always-run)

(rule ((= expr (Single x)) 
       (= sum (expr_size x))) 
      ((set (expr_size expr) sum)) :ruleset always-run)

(rule ((= expr (Switch pred inputs branches)) 
       (= sum  (+ (expr_size inputs) (+ (list_expr_size branches) (expr_size pred)))))
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((= expr (If pred inputs then else)) 
       (= sum (+ (expr_size inputs) (+ (expr_size else) (+ (expr_size then) (expr_size pred))))))
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((= expr (DoWhile in pred-and-output)) 
       (= sum (+ (expr_size pred-and-output) (expr_size in)))) 
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((= expr (Arg ty assum))) 
      ((set (expr_size expr) 1)) :ruleset always-run)

(rule ((= expr (Call func arg)) 
       (= sum (expr_size arg))) 
      ((set (expr_size expr) (+ sum 1))) :ruleset always-run)

(rule ((Empty ty assum)) ((set (expr_size (Empty ty assum)) 0))  :ruleset always-run)

(rule ((= expr (Cons hd tl)) 
       (= sum (+ (list_expr_size tl) (expr_size hd)))) 
      ((set (list_expr_size expr) sum)) :ruleset always-run)

(rule ((Nil)) 
      ((set (list_expr_size (Nil)) 0))  :ruleset always-run)

(rule ((= expr (Alloc id e state ty)) ;; do state edge's expr should be counted?
        (= sum (expr_size e))) 
        ((set (expr_size expr) (+ sum 1))) :ruleset always-run)"#;
