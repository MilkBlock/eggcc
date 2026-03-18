pub(crate) fn fragment() -> String {
    let schema = inject_expr_section(SCHEMA_EGGLOG, &super::schema_dsl::expr_section());
    let schema = inject_list_expr_section(&schema, "");
    let schema = inject_types_section(&schema, "");
    let schema = inject_assumptions_section(&schema, "");
    let schema = inject_constants_section(&schema, "");
    let schema = remove_leaf_node_expr_constructors(&schema);
    let schema = inject_operators_section(&schema, "");
    let schema = inject_tuple_operations_section(&schema, "");
    let schema = inject_control_flow_section(&schema, "");
    let schema = inject_program_type_section(&schema, &super::schema_dsl::program_type_section());
    inject_terms_section(&schema, &super::schema_dsl::terms_section())
}

const SCHEMA_EGGLOG: &str = r#"; Every term is an `Expr` or a `ListExpr`.
(datatype Expr)
; Used for constructing a list of branches for `Switch`es
; or a list of functions in a `Program`.
(datatype ListExpr (Cons Expr ListExpr) (Nil))

; =================================
; Types
; =================================

(sort TypeList)

(datatype BaseType
  (IntT)
  (BoolT)
  (FloatT)
  ; a pointer to a memory region with a particular type
  (PointerT BaseType)
  (StateT))


(datatype Type
  ; a primitive type
  (Base BaseType)
  ; a typed tuple. Use an empty tuple as a unit type.
  ; state edge also has unit type
  (TupleT TypeList)
)

; use TmpType for helpers where the type doesn't matter
; these shouldn't appear in values in the program, only intermediate terms (such as in ivt.egg permutations)
(constructor TmpType () Type)

(constructor TNil () TypeList)
(constructor TCons (BaseType TypeList) TypeList) ; Head element should never be a tuple


; =================================
; Assumptions
; =================================

(datatype Assumption
  ; Assume nothing
  (InFunc String)
  ; The term is in a loop with `input` and `pred_output`.
  ; InLoop is a special context because it describes the argument of the loop. It is a *scope context*.
  ;      input    pred_output
  (InLoop Expr     Expr)
  ; Branch of the switch, and what the predicate is, and what the input is
  (InSwitch i64 Expr Expr)
  ; If the predicate was true, and what the predicate is, and what the input is
  (InIf bool Expr Expr)
)



; =================================
; Leaf nodes
; Constants, argument, and empty tuple
; =================================

; Only a single argument is bound- if multiple values are needed, arg will be a tuple.
; e.g. `(Get (Arg tuple_type) 1)` gets the second value in the argument with some tuple_type.
(constructor Arg (Type Assumption) Expr)

; Constants
(datatype Constant
  (Int i64)
  (Bool bool)
  (Float f64))
; All leaf nodes need the type of the argument
; Type is the type of the bound argument in scope
(constructor Const (Constant Type Assumption) Expr)

; An empty tuple.
; Type is the type of the bound argument in scope
(constructor Empty (Type Assumption) Expr)


; =================================
; Operators
; =================================

(datatype TernaryOp
  ; given a pointer, value, and a state edge
  ; writes the value to the pointer and returns
  ; the resulting state edge
  (Write)
  (Select))
(datatype BinaryOp
  ;; Bitwise operators
  (Bitand)
  ;; integer operators
  (Add)
  (Sub)
  (Div)
  (Mul)
  (LessThan)
  (GreaterThan)
  (LessEq)
  (GreaterEq)
  (Eq)
  (Smin)
  (Smax)
  (Shl)
  (Shr)
  ;; float operators 
  (FAdd)
  (FSub)
  (FDiv)
  (FMul)
  (FLessThan)
  (FGreaterThan) 
  (FLessEq)
  (FGreaterEq)
  (FEq)
  (Fmin)
  (Fmax)
  ;; logical operators
  (And)
  (Or)
  ; given a pointer and a state edge
  ; loads the value at the pointer and returns (value, state edge)
  (Load)
  ; Takes a pointer and an integer, and offsets
  ; the pointer by the integer
  (PtrAdd)
  ; given and value and a state edge, prints the value as a side-effect
  ; the value must be a base value, not a tuple
  ; returns an empty tuple
  (Print)
  ; given a pointer and state edge, frees the whole memory region at the pointer
  (Free))
(datatype UnaryOp
  (Neg)
  (Abs)
  (Not))

; Operators
(constructor Top   (TernaryOp Expr Expr Expr) Expr)
(constructor Bop   (BinaryOp Expr Expr) Expr)
(constructor Uop   (UnaryOp Expr) Expr)
; gets from a tuple. static index
(constructor Get   (Expr i64) Expr)
; (Alloc id amount state_edge pointer_type)
; allocate an integer amount of memory for a particular type
; returns (pointer to the allocated memory, state edge)
(constructor Alloc (i64 Expr Expr BaseType)      Expr)
;               name of func   arg
(constructor Call (String         Expr) Expr)



; =================================
; Tuple operations
; =================================

; `Empty`, `Single` and `Concat` create tuples.
; 1. Use `Empty` for an empty tuple.
; 2. Use `Single` for a tuple with one element.
; 3. Use `Concat` to append the elements from two tuples together.
; Nested tuples are not allowed.


; A tuple with a single element.
; Necessary because we only use `Concat` to add to tuples.
(constructor Single (Expr) Expr)
; Concat appends the elemnts from two tuples together
; e.g. (Concat (Concat (Single a) (Single b))
;              (Concat (Single c) (Single d))) = (a, b, c, d)
;                 expr1       expr2
(constructor Concat (Expr        Expr)       Expr)



; =================================
; Control flow
; =================================

; Switch on a list of lazily-evaluated branches.
; pred must be an integer
;                 pred  inputs   branches     chosen
(constructor Switch (Expr  Expr     ListExpr)    Expr)
; If is like switch, but with a boolean predicate
;             pred inputs   then else
(constructor If (Expr Expr     Expr Expr) Expr)


; A do-while loop.
; Evaluates the input, then evaluates the body.
; Keeps looping while the predicate is true.
; input must have the same type as (output1, output2, ..., outputi)
; input must be a tuple 
; pred must be a boolean
; pred-and-body must be a flat tuple (pred, out1, out2, ..., outi)
; input must be the same type as (out1, out2, ..., outi)
;                  input   pred-and-body
(constructor DoWhile (Expr    Expr)                   Expr)


; =================================
; Top-level expressions
; =================================
(sort ProgramType)
; An entry function and a list of additional functions.
;                      entry function     other functions
(constructor Program     (Expr               ListExpr) ProgramType)
;                   name   input ty  output ty  output
(constructor Function (String Type      Type       Expr)      Expr)

; to get the type of a funciton, look in this table
; since we might not be optimizing the entire program
(relation FunctionHasType (String Type Type))

; Rulesets
(ruleset always-run)
(ruleset is-resolved)
(ruleset error-checking)
(ruleset memory)
(ruleset memory-helpers)
(ruleset smem)

;; Initliazation
(relation bop->string (BinaryOp String))
(relation uop->string (UnaryOp String))
(relation top->string (TernaryOp String))
(bop->string (Add) "Add")
(bop->string (Sub) "Sub")
(bop->string (Div) "Div")
(bop->string (Mul) "Mul")
(bop->string (LessThan) "LessThan")
(bop->string (GreaterThan) "GreaterThan")
(bop->string (LessEq) "LessEq")
(bop->string (GreaterEq) "GreaterEq")
(bop->string (Eq) "Eq")
(bop->string (FAdd) "FAdd")
(bop->string (FSub) "FSub")
(bop->string (FDiv) "FDiv")
(bop->string (FMul) "FMul")
(bop->string (FLessThan) "FLessThan")
(bop->string (FGreaterThan) "FGreaterThan")
(bop->string (FLessEq) "FLessEq")
(bop->string (FGreaterEq) "FGreaterEq")
(bop->string (FEq) "FEq")
(bop->string (And) "And")
(bop->string (Or) "Or")
(bop->string (Load) "Load")
(bop->string (PtrAdd) "PtrAdd")
(bop->string (Print) "Print")
(bop->string (Free) "Free")

;; If anything is put in the DebugExpr relation, we'll extract them instead of the original program.
;; These can then be visualized using the `optimized-rvsdg` run mode
(relation DebugExpr (Expr))

; TERMS
(datatype Term)
(datatype ListTerm (TermCons Term ListTerm) (TermNil))

; TODO: Will probably need ctx so that we can resubstitute?
; (datatype TermAssumption
;   ; Assume nothing
;   (InFunc String)
;   ; The term is in a loop with `input` and `pred_output`.
;   ; InLoop is a special context because it describes the argument of the loop. It is a *scope context*.
;   ;      input    pred_output
;   (InLoop Term     Term)
;   ; Branch of the switch, and what the predicate is, and what the input is
;   (InSwitch i64 Term Term)
;   ; If the predicate was true, and what the predicate is, and what the input is
;   (InIf bool Term Term)
; )

(constructor TermArg () Term)

(constructor TermConst (Constant) Term)

(constructor TermEmpty () Term)

; Term Operators
(constructor TermTop (TernaryOp Term Term Term) Term)
(constructor TermBop (BinaryOp Term Term) Term)
(constructor TermUop (UnaryOp Term) Term)
(constructor TermGet (Term i64) Term)
(constructor TermAlloc (i64 Term Term BaseType) Term)
(constructor TermCall (String Term) Term)

; Tuple Operators
(constructor TermSingle (Term) Term)
(constructor TermConcat (Term Term) Term)

; Control Flow (TODO? Not sure if needed)
; (constructor TermSwitch (Term Term ListTerm) Term)
; (constructor TermIf (Term Term Term Term) Term)

; (constructor TermDoWhile (Term Term) Term)

;;                      inputs, outputs -> number of iterations
;; The minimum possible guess is 1 because of do-while loops
(function LoopNumItersGuess (Expr Expr) i64 :merge (max 1 (min old new)))


;; A hint for no-context mode that this rule
;; fundamentally relies on context and can't be fixed using dummy contexts
(relation RELIESONCONTEXT ())
;; A dummy context for use in no context mode
(let DUMMYCTX (InFunc "DUMMY"))

(ruleset never)"#;

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
    const TUPLE_OPERATIONS_HEADER: &str = r#"; =================================
; Tuple operations
; =================================

"#;

    let header_start = schema
        .find(OPERATORS_HEADER)
        .expect("schema.egg must contain the Operators section header");
    let body_start = header_start + OPERATORS_HEADER.len();
    let tuple_operations_header_start = schema[body_start..]
        .find(TUPLE_OPERATIONS_HEADER)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Tuple operations section header");

    assert!(
        tuple_operations_header_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[tuple_operations_header_start..]);
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

fn inject_program_type_section(schema: &str, replacement: &str) -> String {
    const TOP_LEVEL_EXPRESSIONS_HEADER: &str = r#"; =================================
; Top-level expressions
; =================================
"#;
    const FUNCTION_HAS_TYPE_RELATION: &str = "(relation FunctionHasType";

    let header_start = schema
        .find(TOP_LEVEL_EXPRESSIONS_HEADER)
        .expect("schema.egg must contain the Top-level expressions header");
    let body_start = header_start + TOP_LEVEL_EXPRESSIONS_HEADER.len();
    let function_has_type_relation_start = schema[body_start..]
        .find(FUNCTION_HAS_TYPE_RELATION)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the FunctionHasType relation");

    assert!(
        function_has_type_relation_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[function_has_type_relation_start..]);
    out
}

fn inject_control_flow_section(schema: &str, replacement: &str) -> String {
    const CONTROL_FLOW_HEADER: &str = r#"; =================================
; Control flow
; =================================

"#;
    const TOP_LEVEL_EXPRESSIONS_HEADER: &str = r#"; =================================
; Top-level expressions
; =================================
"#;

    let header_start = schema
        .find(CONTROL_FLOW_HEADER)
        .expect("schema.egg must contain the Control flow section header");
    let body_start = header_start + CONTROL_FLOW_HEADER.len();
    let top_level_header_start = schema[body_start..]
        .find(TOP_LEVEL_EXPRESSIONS_HEADER)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Top-level expressions section header");

    assert!(
        top_level_header_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[top_level_header_start..]);
    out
}

fn inject_tuple_operations_section(schema: &str, replacement: &str) -> String {
    const TUPLE_OPERATIONS_HEADER: &str = r#"; =================================
; Tuple operations
; =================================

"#;
    const CONTROL_FLOW_HEADER: &str = r#"; =================================
; Control flow
; =================================

"#;

    let header_start = schema
        .find(TUPLE_OPERATIONS_HEADER)
        .expect("schema.egg must contain the Tuple operations section header");
    let body_start = header_start + TUPLE_OPERATIONS_HEADER.len();
    let control_flow_header_start = schema[body_start..]
        .find(CONTROL_FLOW_HEADER)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Control flow section header");

    assert!(
        control_flow_header_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[control_flow_header_start..]);
    out
}

fn inject_terms_section(schema: &str, replacement: &str) -> String {
    const TERMS_MARKER: &str = "; TERMS\n";
    const LOOP_NUM_ITERS_GUESS_FUNCTION: &str = "(function LoopNumItersGuess";

    let marker_start = schema
        .find(TERMS_MARKER)
        .expect("schema.egg must contain the Terms marker");
    let body_start = marker_start + TERMS_MARKER.len();
    let loop_num_iters_guess_start = schema[body_start..]
        .find(LOOP_NUM_ITERS_GUESS_FUNCTION)
        .map(|idx| idx + body_start)
        .expect("schema.egg must contain the Terms section boundary header");

    assert!(
        loop_num_iters_guess_start >= body_start,
        "schema.egg section ordering is unexpected"
    );

    let mut out = String::new();
    out.push_str(&schema[..body_start]);

    if !replacement.is_empty() {
        out.push_str("; (Generated from eggplant DSL: src/eggplant_backend/schema_dsl.rs)\n");
        out.push_str(replacement.trim_end());
        out.push_str("\n\n");
    }

    out.push_str(&schema[loop_num_iters_guess_start..]);
    out
}
