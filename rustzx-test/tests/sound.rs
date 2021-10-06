use expect_test::expect;
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

#[test]
fn sound_48k() {
    let mut tester = RustZXTester::new("sound_48k", presets::settings_48k());
    tester.load_sna("sound.48k.sna.gz");
    tester.start_sound_capture();
    tester.emulate_for(Duration::from_secs(2));
    tester.expect_sound(
        "beeper_plus_ay",
        expect![[r#"bEAIV7LVE7591l6VUrCE1ySbEyBSNeVwLjWhDm9Jlpc="#]],
    );
}

#[test]
fn sound_128k() {
    let mut tester = RustZXTester::new("sound_128k", presets::settings_128k());
    tester.load_sna("sound.128k.sna.gz");
    tester.start_sound_capture();
    tester.emulate_for(Duration::from_secs(2));
    tester.expect_sound(
        "beeper_plus_ay",
        expect![[r#"u8WCHu89dFvnMInLGaDFV4ha6FatBtXLJ6szqiUg+ys="#]],
    );
}
