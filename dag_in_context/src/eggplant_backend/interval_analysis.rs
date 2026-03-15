pub(crate) fn fragment() -> String {
    let mut out = String::new();
    out.push_str(GENERATED_MARKER);
    out.push_str(INTERVAL_ANALYSIS);
    out.push('\n');
    out
}

const GENERATED_MARKER: &str =
    "; (Generated from eggplant Rust: src/eggplant_backend/interval_analysis.rs)\n";
const INTERVAL_ANALYSIS: &str = include_str!("../interval_analysis.egg");
