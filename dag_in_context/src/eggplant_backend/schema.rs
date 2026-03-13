pub(crate) fn fragment() -> String {
    let schema = inject_expr_section(SCHEMA_EGGLOG, &super::schema_dsl::expr_section());
    let schema = inject_list_expr_section(&schema, &super::schema_dsl::list_expr_section());
    // NOTE: `expr_section()` emits a combined `(datatypes ...)` block that includes
    // Types/Assumptions/Constants/Operators to avoid forward-reference failures.
    // Remove the original sections so we don't redefine those datatypes later.
    let schema = inject_types_section(&schema, "");
    let schema = inject_assumptions_section(&schema, "");
    let schema = inject_constants_section(&schema, "");
    let schema = remove_leaf_node_expr_constructors(&schema);
    let schema = inject_operators_section(&schema, "");
    let schema = remove_operator_expr_constructors(&schema);
    let schema = inject_program_type_section(&schema, &super::schema_dsl::program_type_section());
    inject_terms_section(&schema, &super::schema_dsl::terms_section())
}

const SCHEMA_EGGLOG: &str = include_str!("../schema.egg");

fn inject_expr_section(schema: &str, replacement: &str) -> String {
    const EXPR_DECL_HEADER: &str = "; Every term is an `Expr` or a `ListExpr`.\n";
    const LIST_EXPR_HEADER: &str = r#"; Used for constructing a list of branches for `Switch`es
; or a list of functions in a `Program`.
"#;

    let header_start = schema
        .find(EXPR_DECL_HEADER)
        .expect("schema.egg must contain the Expr declaration header");
    let body_start = header_start + EXPR_DECL_HEADER.len();
    let list_expr_header_start = schema[body_start..]
        .find(LIST_EXPR_HEADER)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the ListExpr section header");

    assert!(
        list_expr_header_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n");
    }

    out.push_str(&schema[list_expr_header_start..]);
    out
}

fn inject_list_expr_section(schema: &str, replacement: &str) -> String {
    const LIST_EXPR_HEADER: &str = r#"; Used for constructing a list of branches for `Switch`es
; or a list of functions in a `Program`.
"#;
    const TYPES_HEADER: &str = r#"; =================================
; Types
; =================================

"#;

    let header_start = schema
        .find(LIST_EXPR_HEADER)
        .expect("schema.egg must contain the ListExpr section header");
    let body_start = header_start + LIST_EXPR_HEADER.len();
    let types_header_start = schema[body_start..]
        .find(TYPES_HEADER)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Types section header");

    assert!(
        types_header_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[types_header_start..]);
    out
}

fn inject_types_section(schema: &str, replacement: &str) -> String {
    const TYPES_HEADER: &str = r#"; =================================
; Types
; =================================

"#;
    const ASSUMPTIONS_HEADER: &str = r#"; =================================
; Assumptions
; =================================

"#;

    let types_header_start = schema
        .find(TYPES_HEADER)
        .expect("schema.egg must contain the Types section header");
    let types_body_start = types_header_start + TYPES_HEADER.len();
    let assumptions_header_start = schema
        .find(ASSUMPTIONS_HEADER)
        .expect("schema.egg must contain the Assumptions section header");

    assert!(
        assumptions_header_start >= types_body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..types_body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[assumptions_header_start..]);
    out
}

fn inject_operators_section(schema: &str, replacement: &str) -> String {
    const OPERATORS_HEADER: &str = r#"; =================================
; Operators
; =================================

"#;
    const OPERATORS_CONSTRUCTORS_START: &str = r#"; Operators
(constructor Top"#;

    let header_start = schema
        .find(OPERATORS_HEADER)
        .expect("schema.egg must contain the Operators section header");
    let body_start = header_start + OPERATORS_HEADER.len();
    let constructors_start = schema[body_start..]
        .find(OPERATORS_CONSTRUCTORS_START)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Operators constructors section");

    assert!(
        constructors_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[constructors_start..]);
    out
}

fn inject_assumptions_section(schema: &str, replacement: &str) -> String {
    const ASSUMPTIONS_HEADER: &str = r#"; =================================
; Assumptions
; =================================

"#;
    const LEAF_NODES_HEADER: &str = r#"; =================================
; Leaf nodes
; Constants, argument, and empty tuple
; =================================

"#;

    let assumptions_header_start = schema
        .find(ASSUMPTIONS_HEADER)
        .expect("schema.egg must contain the Assumptions section header");
    let assumptions_body_start = assumptions_header_start + ASSUMPTIONS_HEADER.len();
    let leaf_nodes_header_start = schema
        .find(LEAF_NODES_HEADER)
        .expect("schema.egg must contain the Leaf nodes section header");

    assert!(
        leaf_nodes_header_start >= assumptions_body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..assumptions_body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[leaf_nodes_header_start..]);
    out
}

fn inject_constants_section(schema: &str, replacement: &str) -> String {
    const CONSTANTS_MARKER: &str = "; Constants\n";
    const CONST_CONSTRUCTOR_COMMENT: &str = "; All leaf nodes need the type of the argument\n";

    let marker_start = schema
        .find(CONSTANTS_MARKER)
        .expect("schema.egg must contain the Constants marker");
    let body_start = marker_start + CONSTANTS_MARKER.len();
    let constructor_comment_start = schema[body_start..]
        .find(CONST_CONSTRUCTOR_COMMENT)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Constant section boundary comment");

    assert!(
        constructor_comment_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n");
    }

    out.push_str(&schema[constructor_comment_start..]);
    out
}

fn remove_leaf_node_expr_constructors(schema: &str) -> String {
    const ARG_CONSTRUCTOR: &str = "(constructor Arg (Type Assumption) Expr)\n";
    const CONST_CONSTRUCTOR: &str = "(constructor Const (Constant Type Assumption) Expr)\n";
    const EMPTY_CONSTRUCTOR: &str = "(constructor Empty (Type Assumption) Expr)\n";

    let mut out = schema.to_owned();
    for constructor in [ARG_CONSTRUCTOR, CONST_CONSTRUCTOR, EMPTY_CONSTRUCTOR] {
        if out.contains(constructor) {
            out = out.replacen(constructor, "", 1);
        } else {
            panic!("schema.egg must contain the leaf Expr constructor:\n{constructor}");
        }
    }

    out
}

fn remove_operator_expr_constructors(schema: &str) -> String {
    const TOP_CONSTRUCTOR: &str = "(constructor Top   (TernaryOp Expr Expr Expr) Expr)\n";
    const BOP_CONSTRUCTOR: &str = "(constructor Bop   (BinaryOp Expr Expr) Expr)\n";
    const UOP_CONSTRUCTOR: &str = "(constructor Uop   (UnaryOp Expr) Expr)\n";

    let mut out = schema.to_owned();
    for constructor in [TOP_CONSTRUCTOR, BOP_CONSTRUCTOR, UOP_CONSTRUCTOR] {
        if out.contains(constructor) {
            out = out.replacen(constructor, "", 1);
        } else {
            panic!("schema.egg must contain the operator Expr constructor:\n{constructor}");
        }
    }

    out
}

fn inject_program_type_section(schema: &str, replacement: &str) -> String {
    const TOP_LEVEL_EXPRESSIONS_HEADER: &str = r#"; =================================
; Top-level expressions
; =================================
"#;
    const FUNCTION_CONSTRUCTOR_START: &str = "(constructor Function";

    let header_start = schema
        .find(TOP_LEVEL_EXPRESSIONS_HEADER)
        .expect("schema.egg must contain the Top-level expressions header");
    let body_start = header_start + TOP_LEVEL_EXPRESSIONS_HEADER.len();
    let function_constructor_start = schema[body_start..]
        .find(FUNCTION_CONSTRUCTOR_START)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Function constructor");

    assert!(
        function_constructor_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[function_constructor_start..]);
    out
}

fn inject_terms_section(schema: &str, replacement: &str) -> String {
    const TERMS_MARKER: &str = "; TERMS\n";
    const TERM_ASSUMPTION_TODO_COMMENT: &str =
        "; TODO: Will probably need ctx so that we can resubstitute?\n";

    let marker_start = schema
        .find(TERMS_MARKER)
        .expect("schema.egg must contain the Terms marker");
    let body_start = marker_start + TERMS_MARKER.len();
    let todo_comment_start = schema[body_start..]
        .find(TERM_ASSUMPTION_TODO_COMMENT)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Terms section boundary comment");

    assert!(
        todo_comment_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[todo_comment_start..]);
    out
}
