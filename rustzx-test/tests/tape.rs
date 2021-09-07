use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

#[test]
fn fastload_test() {
    let mut tester = RustZXTester::new("fastload_test", presets::settings_48k_nosound());
    tester.load_tape("simple_tape.tap");
    tester.emulate_for(Duration::from_millis(45));
    tester.expect_screen("screen_1.png");
    tester.emulate_for(Duration::from_millis(10));
    tester.expect_screen("screen_2.png");
    tester.expect_border("border_1.png");
}
