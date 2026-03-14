use egglog::ast::{Command, Schema, Span};

pub(crate) fn fragment() -> String {
    inject_generated_prefix(UTILITY_EGGLOG, &generated_declarations_section())
}

const UTILITY_EGGLOG: &str = include_str!("../utility/util.egg");
const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/util.rs)\n";
const LEADING_EXPRESSIONS_COMMENT: &str =
    ";; Leading expressions are tuples that are used as a whole\n";
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

fn inject_generated_prefix(utility: &str, replacement: &str) -> String {
    let raw_start = utility
        .find(LEADING_EXPRESSIONS_COMMENT)
        .expect("utility/util.egg must contain the leading-expressions boundary anchor");

    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(replacement);
    out.push_str("\n\n");
    out.push_str(&utility[raw_start..]);
    out
}
