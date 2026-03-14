use egglog::ast::{Command, Schema, Span};

pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(&generated_declarations_section());
    out.push('\n');
    out
}

const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/util.rs)\n";
const GENERATED_PREFIX_RULES_AND_RELATIONS: &str = r#"
(rule ((Switch pred inputs branch)) ((union (ListExpr-suffix branch 0) branch)) :ruleset always-run)

(rule ((= (ListExpr-suffix top n) (Cons hd tl)))
    ((union (ListExpr-ith top n) hd)
     (union (ListExpr-suffix top (+ n 1)) tl)) :ruleset always-run)

(rule ((= (ListExpr-suffix list n) (Nil)))
    ((set (ListExpr-length list) n)) :ruleset always-run)

(rewrite (Append (Cons a b) e)
   (Cons a (Append b e))
   :ruleset always-run)
(rewrite (Append (Nil) e)
   (Cons e (Nil))
   :ruleset always-run)

(rule ((HasType expr (TupleT tl))
       (= len (TypeList-length tl)))
      ((set (tuple-length expr) len)) :ruleset always-run)


;; Leading expressions are tuples that are used as a whole
;; during optimization and are matched in the query.
(relation leading-Expr (Expr))
(relation leading-Expr-list (ListExpr))

(rule ((= e (DoWhile inputs pred_out)))
      ((leading-Expr e)
       (leading-Expr inputs)
       (leading-Expr pred_out))
      :ruleset always-run)
(rule ((= e (If cond inputs thn els)))
       ((leading-Expr e)
        (leading-Expr inputs)
        (leading-Expr thn)
        (leading-Expr els))
       :ruleset always-run)
(rule ((= e (Switch pred inputs branch)))
      ((leading-Expr e)
       (leading-Expr-list branch)
       (leading-Expr inputs))
       :ruleset always-run)
(rule ((leading-Expr-list (Cons hd tl)))
      ((leading-Expr hd)
       (leading-Expr-list tl))
      :ruleset always-run)
(rule ((= e (Arg t a)))
      ((leading-Expr e))
      :ruleset always-run)

;; Create a Get for everything that is a leading expression, so not everything needs a Get node
(rule ((Single expr)) ((union (Get (Single expr) 0) expr)) :ruleset always-run)
;; initial get
(rule ((leading-Expr tuple)
       (> (tuple-length tuple) 0))
      ((Get tuple 0))
      :ruleset always-run)
;; next get
(rule ((leading-Expr tuple)
       (= len (tuple-length tuple))
       (= ith (Get tuple i))
       (< (+ i 1) len)
       )
       ((Get tuple (+ 1 i)))
       :ruleset always-run)

;; push Get through Concat in a way that isn't quadratic
;; add-gets to      original-expr   current-expr    start-index
(relation Add-Gets (Expr            Expr            i64))
(rule ((Get x i))
      ((Add-Gets x x 0))
      :ruleset always-run)
(rule ((Add-Gets orig (Concat left right) n)
       (= len (tuple-length left)))
      ((Add-Gets orig left n)
       (Add-Gets orig right (+ n len)))
      :ruleset always-run)

(rule ((Add-Gets orig (Single e) n))
      ((union (Get orig n) e))
      :ruleset always-run)

;; now, for things other than Concat we Add-All-Gets
;; put things here (potentially tuples) that are not
;; just an eclass with a Concat
(relation Not-Just-Concat (Expr))
;; original-expr, current-expr, offset, current-pos
(relation Add-All-Gets (Expr Expr i64 i64))
(rule ((Add-Gets orig something n)
       (Not-Just-Concat something))
      ((Add-All-Gets orig something n 0))
      :ruleset always-run)

(rule ((Add-All-Gets orig something offset pos)
       (< pos (tuple-length something)))
      ((union (Get orig (+ offset pos)) (Get something pos))
       (Add-All-Gets orig something offset (+ pos 1)))
      :ruleset always-run)

(rule ((= lhs (Arg a b)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)
(rule ((= lhs (Top a b c d)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)
(rule ((= lhs (Bop a b c)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)
(rule ((= lhs (Call a b)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)
(rule ((= lhs (Uop a b)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)
(rule ((= lhs (Switch a b c)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)
(rule ((= lhs (If a b c d)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)
(rule ((= lhs (DoWhile a b)))
      ((Not-Just-Concat lhs))
      :ruleset always-run)


;; A temporary context.
;; Be sure to delete at the end of all actions or else!!!
;; This is safer than using a persistant context, since we may miss an important part of the query.
(rule ((TmpCtx))
  ((panic "TmpCtx should not exist outside rule body"))
  :ruleset always-run)


(ruleset subsume-after-helpers)
;; After running the `saturating` ruleset, these if statements can be subsumed
(relation ToSubsumeIf (Expr Expr Expr Expr))
;; Workaround of https://github.com/egraphs-good/egglog/issues/462
;; Make sure the if we are subsuming is present
(rule ((ToSubsumeIf a b c d)
       (If a b c d))
      ((subsume (If a b c d)))
      :ruleset subsume-after-helpers)

(ruleset add-to-debug-expr)
"#;

fn generated_declarations_section() -> String {
    // Grow the generated util prefix one self-contained chunk at a time,
    // while leaving the remaining raw egglog text below a stable anchor.
    [
        Command::Function {
            span: Span::Panic,
            name: "ListExpr-length".into(),
            schema: schema(&["ListExpr"], "i64"),
            merge: None,
        },
        Command::Constructor {
            span: Span::Panic,
            name: "ListExpr-ith".into(),
            schema: schema(&["ListExpr", "i64"], "Expr"),
            cost: None,
            unextractable: true,
        },
        Command::Constructor {
            span: Span::Panic,
            name: "ListExpr-suffix".into(),
            schema: schema(&["ListExpr", "i64"], "ListExpr"),
            cost: None,
            unextractable: true,
        },
        Command::Constructor {
            span: Span::Panic,
            name: "Append".into(),
            schema: schema(&["ListExpr", "Expr"], "ListExpr"),
            cost: None,
            unextractable: true,
        },
        Command::Function {
            span: Span::Panic,
            name: "tuple-length".into(),
            schema: schema(&["Expr"], "i64"),
            merge: None,
        },
        Command::Constructor {
            span: Span::Panic,
            name: "TmpCtx".into(),
            schema: schema(&[], "Assumption"),
            cost: None,
            unextractable: false,
        },
    ]
    .into_iter()
    .map(|command| command.to_string())
    .collect::<Vec<_>>()
    .join("\n")
        + "\n"
        + GENERATED_PREFIX_RULES_AND_RELATIONS
}

fn schema(inputs: &[&str], output: &str) -> Schema {
    Schema {
        input: inputs.iter().map(|sort| (*sort).into()).collect(),
        output: output.into(),
    }
}
