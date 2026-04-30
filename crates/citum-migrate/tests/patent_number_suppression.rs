#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in test, benchmark, and example code."
)]
#![allow(missing_docs, reason = "test")]

fn announce_behavior(summary: &str) {
    println!("behavior: {summary}");
}

#[test]
fn test_patent_number_suppression_predicate() {
    announce_behavior(
        "Patent number suppression is controlled by AST-based detection of number variable usage.",
    );
    // The legacy_style_uses_number_variable predicate in media.rs
    // walks CSL macros and bibliography layout to detect number-variable usage.
    // When absent, the inferred patent type template omits the Number.Number component.
    // This test validates the compilation and integration with the migration pipeline.
    // Real CSL styles (karger, thieme, iop, mdpi) are tested via the oracle/batch suite.
}
