use trybuild::TestCases;

#[test]
fn ui() {
    let testcase = TestCases::new();
    testcase.compile_fail("tests/ui/fail/*.rs");
    testcase.pass("tests/ui/pass/*.rs");
}
