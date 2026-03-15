pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(SWAP_IF);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/swap_if.rs)\n";
const SWAP_IF: &str = r#"(ruleset swap-if)

;; swaps the order of the then and else branches
;; in an if using Not

(rule
  ((= lhs (If pred inputs then else)))
  (
    (union lhs (If (Uop (Not) pred) inputs else then))
  )
  :ruleset swap-if)


;; for if statements with two outputs, swaps the order
;; of the outputs
(rule
  ((= lhs (If pred inputs then else))
   (= (tuple-length then) 2)
   (= (tuple-length else) 2))
  (
    (union
      (Concat (Single (Get lhs 1)) (Single (Get lhs 0)))
      (If pred inputs
          (Concat (Single (Get then 1)) (Single (Get then 0)))
          (Concat (Single (Get else 1)) (Single (Get else 0)))))
  )
  :ruleset swap-if)"#;
