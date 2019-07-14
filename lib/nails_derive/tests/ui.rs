#[test]
fn ui() {
    if option_env!("CARGO")
        .unwrap_or("cargo")
        .ends_with("cargo-tarpaulin")
    {
        eprintln!(
            "Skipping ui tests to avoid incompatibility between cargo-tarpaulin and trybuild"
        );
        return;
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
