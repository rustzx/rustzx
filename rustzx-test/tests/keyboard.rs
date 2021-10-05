use expect_test::expect;
use rustzx_core::{
    zx::keys::{CompoundKey, ZXKey},
    IterableEnum,
};
use rustzx_test::framework::{presets, RustZXTester};

#[test]
fn standard_keys() {
    let mut t = RustZXTester::new("standard_keys", presets::settings_48k_nosound());
    t.enable_debug_port();
    t.load_sna("keyboard.48k.sna.gz");

    let mut out = String::new();

    // Check emulator initial state (all bits should set to 1)
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();

    let keys = ZXKey::iter().collect::<Vec<_>>();

    // Press all sequential
    for k in &keys {
        t.emulator().send_key(*k, true);
        t.sync_target();
        t.emulate_frame();
        out += &t.debug_port().take_text();
    }

    // Release all sequential
    for k in &keys {
        t.emulator().send_key(*k, false);
        t.sync_target();
        t.emulate_frame();
        out += &t.debug_port().take_text();
    }

    // Expect full key press/release log. Here we just compare hashes because
    // keyboard input log is too big to inline it directly into the test file
    t.expect_text(
        "log",
        out,
        expect![[r#"lF44fsm0VLApiGX1LTQOYRgj40hPrX9L5SaA3nF/i6w="#]],
    );
}

#[test]
fn compound_keys() {
    let mut t = RustZXTester::new("compound_keys", presets::settings_48k_nosound());
    t.enable_debug_port();
    t.load_sna("keyboard.48k.sna.gz");

    let mut out = String::from("SEQUENTIAL\n");

    let keys = CompoundKey::iter().collect::<Vec<_>>();

    let mut prev = None;

    // Separate key presses
    for k in &keys {
        if let Some(prev) = prev {
            t.emulator().send_compound_key(prev, false);
        }
        t.emulator().send_compound_key(*k, true);
        t.sync_target();
        t.emulate_frame();
        out += &t.debug_port().take_text();
        prev = Some(*k);
    }

    // Reset keyboard to its initial state by releasing last pressed key
    // and write this initial state to the log
    out += "INITIAL\n";
    t.emulator().send_compound_key(prev.unwrap(), false);
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();

    // Check that simultaneously pressed compound keys do not cancel each other.
    // resulting keyboard state should be logical "AND" (because keys are inversed)
    out += "SIMULTANEOUS\n";
    t.emulator().send_compound_key(CompoundKey::ArrowLeft, true);
    t.emulator()
        .send_compound_key(CompoundKey::ArrowRight, true);
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();
    t.emulator()
        .send_compound_key(CompoundKey::ArrowLeft, false);
    t.emulator()
        .send_compound_key(CompoundKey::ArrowRight, false);

    // Check that compound keys can be pressed simulateneously with standard keys
    out += "WITH_STANDARD\n";
    t.emulator().send_compound_key(CompoundKey::ArrowLeft, true);
    t.emulator().send_key(ZXKey::N5, true); // overlaps with compound
    t.emulator().send_key(ZXKey::A, true); // unrelated key
    t.sync_target();
    t.emulate_frame();
    // After releasing compound key, overlapping standard key N5 should be still
    // notified by the emualtor as pressed
    t.emulator()
        .send_compound_key(CompoundKey::ArrowLeft, false);
    out += &t.debug_port().take_text();
    t.sync_target();
    t.emulate_frame();
    out += &t.debug_port().take_text();

    // Expect full key press/release log. Here we just compare hashes because
    // keyboard input log is too big to inline it directly into the test file
    t.expect_text(
        "log",
        out,
        expect![[r#"v01HM6RHAtHfvFEnvCXae4dl1FrHEISrnDgljzvMcoE="#]],
    );
}
