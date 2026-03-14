use egglog::ast::{Command, Schema, Span};

pub(crate) fn fragment() -> String {
    inject_generated_prefix(TYPE_ANALYSIS_EGGLOG, &generated_declarations_section())
}

const TYPE_ANALYSIS_EGGLOG: &str = include_str!("../type_analysis.egg");
const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/type_analysis.rs)\n";
const PROPAGATE_ARG_TYPES_COMMENT: &str = "; Propagate arg types up\n";
const HELPER_RULES_AND_RELATIONS: &str = r#"(rewrite (TLConcat (TNil) r) r :ruleset type-helpers)
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
"#;

fn generated_declarations_section() -> String {
    // Start the type_analysis migration with the declaration-only prefix, while
    // keeping the rule bodies in raw egglog until later rounds.
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
        + HELPER_RULES_AND_RELATIONS
}

fn schema(inputs: &[&str], output: &str) -> Schema {
    Schema {
        input: inputs.iter().map(|sort| (*sort).into()).collect(),
        output: output.into(),
    }
}

fn inject_generated_prefix(type_analysis: &str, replacement: &str) -> String {
    let raw_start = type_analysis
        .find(PROPAGATE_ARG_TYPES_COMMENT)
        .expect("type_analysis.egg must contain the arg-type propagation anchor");

    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(replacement);
    out.push_str("\n\n");
    out.push_str(&type_analysis[raw_start..]);
    out
}
