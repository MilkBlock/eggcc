use eggplant::prelude::RunConfig;

pub(crate) fn prologue() -> String {
    // Migration bootstrapping:
    // - Keep the existing egglog text prologue so behavior stays unchanged.
    // - Provide a single compile-time switch point for gradually replacing
    //   `.egg` includes with eggplant Rust code.
    let _ = RunConfig::Once;
    crate::prologue_egglog_text()
}

#[cfg(test)]
mod tests {
    #[test]
    fn eggplant_feature_smoke() {
        let s = super::prologue();
        assert!(!s.is_empty());
    }
}

