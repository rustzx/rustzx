use expect_test::expect;
use rustzx_core::zx::joy::kempston::KempstonKey;
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

#[test]
fn kempston_joy() {
    let mut settings = presets::settings_48k_nosound();
    settings.kempston_enabled = true;
    let mut t = RustZXTester::new("kempston_joy", settings);
    t.enable_debug_port();
    t.load_sna("kempston_joy.48k.sna.gz");

    // Check emulator initial state (all bits should set to 1)
    t.emulate_for(Duration::from_millis(100));
    let test_output = t.debug_port().get_text();
    expect![[r#"00"#]].assert_eq(&test_output);

    use KempstonKey as KK;

    let keys = [
        KK::Right,
        KK::Left,
        KK::Down,
        KK::Up,
        KK::Fire,
        KK::Ext1,
        KK::Ext2,
        KK::Ext3,
    ];

    let mut out = String::new();

    // Press all sequential
    for k in &keys {
        t.emulator().send_kempston_key(*k, true);
        t.sync_target();
        t.emulate_for(Duration::from_millis(20));
        out += &t.debug_port().get_text();
    }

    // Release all sequential
    for k in &keys {
        t.emulator().send_kempston_key(*k, false);
        t.sync_target();
        t.emulate_for(Duration::from_millis(20));
        out += &t.debug_port().get_text();
    }

    expect![[r#"01>03>07>0F>1F>3F>7F>FF>FE>FC>F8>F0>E0>C0>80>00"#]].assert_eq(&out);
}
