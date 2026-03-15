pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(MEMORY);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str = "; (Generated from eggplant Rust: src/eggplant_backend/memory.rs)\n";
const MEMORY: &str = include_str!("../optimizations/memory.egg");
