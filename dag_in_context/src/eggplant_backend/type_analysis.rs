use egglog::ast::{Command, Schema, Span};

pub(crate) fn fragment() -> String {
    inject_generated_prefix(TYPE_ANALYSIS_EGGLOG, &generated_declarations_section())
}

const TYPE_ANALYSIS_EGGLOG: &str = include_str!("../type_analysis.egg");
const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/type_analysis.rs)\n";
const BINARY_OPS_COMMENT: &str = "; Binary ops\n";
const GENERATED_PREFIX_RULES_AND_RELATIONS: &str = r#"(rewrite (TLConcat (TNil) r) r :ruleset type-helpers)
(rewrite (TLConcat (TCons hd tl) r)
         (TCons hd (TLConcat tl r))
         :ruleset type-helpers)

;; Don't match on TypeList-ith because it is now lazily instantiated!
(rule () ((set (TypeList-length (TNil)) 0)) :ruleset type-helpers)
(rule ((= lst (TCons hd tl))
       (= len (TypeList-length tl)))
      ((set (TypeList-length lst) (+ 1 len))) :ruleset type-helpers)
(rewrite (TypeList-ith (TCons hd tl) 0) hd :ruleset type-helpers)
(rewrite (TypeList-ith (TCons hd tl) i) (TypeList-ith tl (- i 1)) 
      :when ((> i 0)) 
      :ruleset type-helpers)

(rule ((TypeList-ith list i)
       (= (TypeList-length list) n)
       (>= i n))
      ((panic "TypeList-ith out of bounds")) :ruleset type-helpers)

(relation HasType (Expr Type))


;; Keep track of type expectations for error messages
(relation ExpectType (Expr Type String))
(rule (
        (ExpectType e expected msg)
        (HasType e actual)
        (!= expected actual) ;; not okay unless we saturate type helpers.
      )
      ((extract "Expecting expression")
       (extract e)
       (extract "to have type")
       (extract expected)
       (extract "but got type")
       (extract actual)
       (extract "with message")
       (extract msg)
       (panic "type mismatch- check RUST_LOG=info for expressions that mismatched"))
      :ruleset error-checking)


(rule ((= (Const c1 ty1 ctx1) (Const c2 ty2 ctx2))
       (= ctx1 (InFunc name))
       (!= c1 c2))
      ((panic "Unsoundness detected: const values differ at top level"))
      :ruleset error-checking)

(relation HasArgType (Expr Type))

(rule ((HasArgType (Arg t1 ctx) t2)
       (!= t1 t2))
      ((panic "arg type mismatch"))
      :ruleset error-checking)

(rule ((= lhs (Function name in out body))
       (HasArgType body ty)
       (HasArgType body ty2)
       (!= ty ty2))
      ((panic "arg type mismatch in function"))
      :ruleset error-checking)

; Propagate arg types up
(rule ((= lhs (Uop _ e))
       (HasArgType e ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Bop _ a b))
       (HasArgType a ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Bop _ a b))
       (HasArgType b ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Top _ a b c))
       (HasArgType a ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Top _ a b c))
       (HasArgType b ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Top _ a b c))
       (HasArgType c ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Get e _))
       (HasArgType e ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Alloc _id e state _))
       (HasArgType e ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Call _ e))
       (HasArgType e ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Single e))
       (HasArgType e ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Concat e1 e2))
       (HasArgType e1 ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Concat e1 e2))
       (HasArgType e2 ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Switch pred inputs (Cons branch rest)))
       (HasArgType pred ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (Switch pred inputs (Cons branch rest)))
       (HasArgType branch ty)
       (HasType inputs ty2)
       (!= ty ty2))
      ((panic "switch branches then branch has incorrect input type"))
      :ruleset error-checking)
;; demand with one fewer branches
(rule ((= lhs (Switch pred inputs (Cons branch rest))))
      ((Switch pred inputs rest))
      :ruleset type-analysis)
(rule ((= lhs (If c i t e))
       (HasArgType c ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)
(rule ((= lhs (If c i t e))
       (HasType i ty)
       (HasArgType t ty2)
       (!= ty ty2))
      ((panic "if branches then branch has incorrect input type"))
      :ruleset error-checking)
(rule ((= lhs (If c i t e))
       (HasType i ty)
       (HasArgType e ty2)
       (!= ty ty2))
      ((panic "if branches else branch has incorrect input type"))
      :ruleset error-checking)


(rule ((= lhs (DoWhile ins body))
       (HasArgType ins ty))
      ((HasArgType lhs ty))
      :ruleset type-analysis)

; Don't push arg types through Program, Function, DoWhile, Let exprs because
; these create new arg contexts.

; Primitives
(rule ((= lhs (Const (Int i) ty ctx)))
      ((HasType lhs (Base (IntT)))
       (HasArgType lhs ty))
      :ruleset type-analysis)

(rule ((= lhs (Const (Bool b) ty ctx)))
      ((HasType lhs (Base (BoolT)))
       (HasArgType lhs ty))
      :ruleset type-analysis)

(rule ((= lhs (Const (Float b) ty ctx)))
      ((HasType lhs (Base (FloatT)))
       (HasArgType lhs ty))
      :ruleset type-analysis)

(rule ((= lhs (Empty ty ctx)))
      ((HasType lhs (TupleT (TNil)))
       (HasArgType lhs ty))
      :ruleset type-analysis)

; Unary Ops
(rule (
        (= lhs (Uop (Not) e))
        (HasType e (Base (BoolT)))
      )
      ((HasType lhs (Base (BoolT))))
      :ruleset type-analysis)
(rule ((= lhs (Uop (Not) e)))
      ((ExpectType e (Base (BoolT)) "(Not)"))
      :ruleset type-analysis)

(rule (
      (= lhs (Uop (Neg) e))
      (HasType e (Base (IntT)))
) (
      (HasType lhs (Base (IntT)))
) :ruleset type-analysis)

(rule (
      (= lhs (Uop (Neg) e))
) (
      (ExpectType e (Base (IntT)) "(Neg)")
) :ruleset type-analysis)

(rule (
        (= lhs (Uop (Abs) e))
        (HasType e (Base (IntT)))
      )
      ((HasType lhs (Base (IntT))))
      :ruleset type-analysis)
(rule ((= lhs (Uop (Abs) e)))
      ((ExpectType e (Base (IntT)) "(Abs)"))
      :ruleset type-analysis)


(rule (
        (= lhs (Bop (Print) e state))
        (HasType e _ty)             ; just make sure it has some type.
      )
      ((HasType lhs (Base (StateT))))
      :ruleset type-analysis)

(rule (
        (= lhs (Bop (Print) e state))
        (HasType e (TupleT ty))
      )
      ((panic "Don't print a tuple"))
      :ruleset error-checking)

(rule ((= lhs (Bop (Free) e s))
       (HasType e (Base (PointerT _ty))))
      ((HasType lhs (Base (StateT))))
      :ruleset type-analysis)
(rule ((= lhs (Bop (Free) e s))
       (HasType e (Base (IntT))))
      ((panic "Free expected pointer, received integer"))
      :ruleset error-checking)
(rule ((= lhs (Bop (Free) e s))
       (HasType e (TupleT _ty)))
      ((panic "Free expected pointer, received tuple"))
      :ruleset error-checking)

(rule (
        (= lhs (Bop (Load) e state))
        (HasType e (Base (PointerT ty)))
      )
      ((HasType lhs (TupleT (TCons ty (TCons (StateT) (TNil))))))
      :ruleset type-analysis)
(rule (
        (= lhs (Bop (Load) e state))
        (HasType e ty)
        (= ty (Base (IntT)))
      )
      ((panic "(Load) expected pointer, received int"))
      :ruleset error-checking)
(rule (
        (= lhs (Bop (Load) e state))
        (HasType e ty)
        (= ty (TupleT x))
      )
      ((panic "(Load) expected pointer, received tuple"))
      :ruleset error-checking)

(rule (
        (= lhs (Top (Select) pred v1 v2))
      )
      ((ExpectType pred (Base (BoolT)) "(Select)"))
      :ruleset type-analysis)

(rule (
        (= lhs (Top (Select) pred v1 v2))
        (HasType v1 ty)
        (HasType v2 ty)
      )
      ((HasType lhs ty))
      :ruleset type-analysis)

(rule (
        (= lhs (Top (Select) pred v1 v2))
        (HasType v1 ty1)
        (HasType v2 ty2)
        (!= ty1 ty2)
      )
      ((panic "(Select) branches had different types"))
      :ruleset error-checking)
"#;

fn generated_declarations_section() -> String {
    // Grow the generated type_analysis prefix one self-contained chunk at a time,
    // while leaving the remaining raw egglog text below a stable anchor.
    [
        Command::AddRuleset("type-analysis".into()),
        Command::AddRuleset("type-helpers".into()),
        Command::Constructor {
            span: Span::Panic,
            name: "TLConcat".into(),
            schema: schema(&["TypeList", "TypeList"], "TypeList"),
            cost: None,
            unextractable: true,
        },
        Command::Function {
            span: Span::Panic,
            name: "TypeList-length".into(),
            schema: schema(&["TypeList"], "i64"),
            merge: None,
        },
        Command::Constructor {
            span: Span::Panic,
            name: "TypeList-ith".into(),
            schema: schema(&["TypeList", "i64"], "BaseType"),
            cost: None,
            unextractable: true,
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

fn inject_generated_prefix(type_analysis: &str, replacement: &str) -> String {
    let raw_start = type_analysis
        .find(BINARY_OPS_COMMENT)
        .expect("type_analysis.egg must contain the binary-ops boundary anchor");

    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(replacement);
    out.push_str("\n\n");
    out.push_str(&type_analysis[raw_start..]);
    out
}
