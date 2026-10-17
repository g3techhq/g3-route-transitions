//! Every error the derive reports, with the message and span a user sees.
//!
//! Regenerate the expected output after an intended change with
//! `TRYBUILD=overwrite cargo test --test compile_fail`, and review the diff.
#[test]
fn derive_errors() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
