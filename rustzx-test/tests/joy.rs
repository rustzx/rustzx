use expect_test::expect;
use rustzx_core::{
    zx::{
        joy::{
            kempston::KempstonKey,
            sinclair::{SinclairJoyNum, SinclairKey},
        },
        keys::{CompoundKey, ZXKey},
    },
    IterableEnum,
};
use rustzx_test::framework::{presets, RustZXTester};

#[test]
fn kempston_joy() {
    let mut settings = presets::settings_48k_nosound();
    settings.kempston_enabled = true;
    let mut t = RustZXTester::new("kempston_joy", settings);
    t.enable_debug_port();
    t.load_sna("kempston_joy.48k.sna.gz");

    let mut out = String::new();

    // Check emulator initial state (all bits should set to 0)
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();

    let keys = KempstonKey::iter().collect::<Vec<_>>();

    // Press all sequential
    for k in &keys {
        t.emulator().send_kempston_key(*k, true);
        t.sync_target();
        t.emulate_frame();
        out += &t.debug_port().take_text();
    }

    // Release all sequential
    for k in &keys {
        t.emulator().send_kempston_key(*k, false);
        t.sync_target();
        t.emulate_frame();
        out += &t.debug_port().take_text();
    }

    expect![[r#"00,01,03,07,0F,1F,3F,7F,FF,FE,FC,F8,F0,E0,C0,80,00,"#]].assert_eq(&out);
}

#[test]
fn sinclair_joy() {
    let mut t = RustZXTester::new("sinclair_joy", presets::settings_48k_nosound());
    t.enable_debug_port();
    // We use keyboard SNA here because technically, sinclair joystick emulates key presses
    t.load_sna("keyboard.48k.sna.gz");

    let mut out = String::new();

    let joys = SinclairJoyNum::iter().collect::<Vec<_>>();
    let buttons = SinclairKey::iter().collect::<Vec<_>>();

    for joy in joys {
        out += &format!("JOY_BUTTONS({:?})\n", joy);
        for button in &buttons {
            t.emulator().send_sinclair_key(joy, *button, true);
            t.sync_target();
            t.emulate_frame();
            out += &t.debug_port().take_text();
            t.emulator().send_sinclair_key(joy, *button, false);
        }
    }

    out += "SIMULTANEOUS\n";
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Fist, SinclairKey::Right, true);
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Second, SinclairKey::Left, true);
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Fist, SinclairKey::Right, false);
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Second, SinclairKey::Left, false);

    out += "WITH_OTHER_KEYS\n";
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Fist, SinclairKey::Fire, true);
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Fist, SinclairKey::Left, true);
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Second, SinclairKey::Fire, true);
    t.emulator().send_key(ZXKey::A, true); // unrelated key
    t.emulator().send_key(ZXKey::N5, true); // overlaps with joy key
    t.emulator().send_compound_key(CompoundKey::Delete, true); // overlaps with joy key (N0)
    t.emulator().send_compound_key(CompoundKey::Break, true); // unrelated compound key
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();
    // Check that joy key release does not clear unrelated keys/compound keys
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Fist, SinclairKey::Fire, false);
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Fist, SinclairKey::Left, false);
    t.emulator()
        .send_sinclair_key(SinclairJoyNum::Second, SinclairKey::Fire, false);
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();

    // Expect full key press/release log. See reasoning about comparing hashes
    // in keyboard.rs
    t.expect_text(
        "log",
        out,
        expect![[r#"F6bYEdfQ8M9gpyCivt2vuKMws83uDEmuB3Q7OQlXucU="#]],
    );
}
