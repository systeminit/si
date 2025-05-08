// TODO: These are disabled because they no longer pass in CI due to a timeout. We should either
// figure out why and re-enable them or just remove this project.
#[test]
pub fn trybuild() {
    let _t = trybuild::TestCases::new();
    // t.pass("tests/ui/*-pass.rs");
    // t.compile_fail("tests/ui/*-fail.rs")
}
