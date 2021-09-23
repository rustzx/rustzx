use expect_test::expect;
use rustzx_core::zx::keys::ZXKey;
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

#[test]
fn no_fastload() {
    let mut settings = presets::settings_48k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("no_fastload", settings);
    tester.load_tap("simple_tape.tap");
    // Wait for ROM to load
    tester.emulate_for(Duration::from_millis(2000));
    // Emulate LOAD ""
    tester.send_keystrokes(
        &[
            &[ZXKey::J],
            &[ZXKey::SymShift, ZXKey::P],
            &[ZXKey::SymShift, ZXKey::P],
            &[ZXKey::Enter],
        ],
        Duration::from_millis(100),
    );

    // Check that tape is not loading until signaled manually
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen(
        "empty",
        expect![[r#"nI+vo8GaRwKwWTPTP2f22Wcgm9nEwMlm16+Cmzird2w="#]],
    );
    tester.expect_border(
        "empty",
        expect![[r#"CkU7FUXUKUZneunabAn/h+88EDIxzvO1aqCl5LadYEs="#]],
    );

    tester.emulator().play_tape();
    tester.emulate_for(Duration::from_millis(2000));

    // Check tack tape is started loading
    tester.expect_border(
        "sync_pulses",
        expect![[r#"Oc++rVrRSea7L5+dCz066kS/mPzhKZ8MhhvVo+8r5iY="#]],
    );

    // Check that data block started loading
    tester.emulate_for(Duration::from_millis(3100));
    tester.expect_border(
        "data_pulses",
        expect![[r#"pf8oQYn3t7yAJuLr0MjOpROdoDJ1wK1CsqQvsasuDew="#]],
    );

    // Check that Loader has been loaded
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen(
        "block_1",
        expect![[r#"+o3MYnfBeDMtimIE/+6+o2/9h1OgtZ8izbO7b/jOiMc="#]],
    );

    // Check that second block has been loaded.
    tester.emulate_for(Duration::from_millis(45000));
    tester.expect_border(
        "end",
        expect![[r#"tmGY7e4h+XA3px6BcqnCXF83NEdBqVw8PW9sQtpMAvM="#]],
    );
    tester.expect_screen(
        "block_2",
        expect![[r#"zDQzdQr19uTYaZouk7ex+pkylk2TRFAuenooMVFjkyQ="#]],
    );
}

#[test]
fn no_fastload_128k() {
    let mut settings = presets::settings_128k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("no_fastload_128k", settings);
    tester.load_tap("simple_tape.tap");
    // Wait for ROM to load
    tester.emulate_for(Duration::from_millis(3000));

    // Emulate Enter keypress to load tape
    tester.send_keystrokes(&[&[ZXKey::Enter]], Duration::from_millis(100));

    // Check that tape is not loading until signaled manually
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen(
        "ready_to_load",
        expect![[r#"jSZUNHDpTpRwuQydjVmchehIKSlgP+bhcKE8bi+yZoc="#]],
    );
    tester.expect_border(
        "ready_to_load",
        expect![[r#"CkU7FUXUKUZneunabAn/h+88EDIxzvO1aqCl5LadYEs="#]],
    );

    tester.emulator().play_tape();
    tester.emulate_for(Duration::from_millis(2000));

    // Check tack tape is started loading
    tester.expect_border(
        "sync_pulses",
        expect![[r#"lbeRI1CgCfyRmOpuVsyzKEzvnnJe/3CZJUj+BEaypwE="#]],
    );

    // Check that data block started loading
    tester.emulate_for(Duration::from_millis(3000));
    tester.expect_border(
        "data_pulses",
        expect![[r#"TCEfW/Ng3an2OgjmHdfYMvSdUVUt3RuEw+kYGgoEIh8="#]],
    );

    // Check that Loader has been loaded
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen(
        "block_1",
        expect![[r#"FfpfX8Nl7RPODebDhyDPqEpNWieSSG7PYBXvg9ty7k0="#]],
    );

    // Switching to the next block is already tested by `no_fastload` test, therefore
    // we can skip this for 128K test
}

#[test]
fn tape_stop() {
    let mut settings = presets::settings_48k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("tape_stop", settings);
    tester.load_tap("simple_tape.tap");
    // Wait for ROM to load
    tester.emulate_for(Duration::from_millis(2000));
    // Emulate LOAD ""
    tester.send_keystrokes(
        &[
            &[ZXKey::J],
            &[ZXKey::SymShift, ZXKey::P],
            &[ZXKey::SymShift, ZXKey::P],
            &[ZXKey::Enter],
        ],
        Duration::from_millis(100),
    );

    tester.emulator().play_tape();
    tester.emulate_for(Duration::from_millis(2000));

    // Check tack tape is started loading
    tester.expect_border(
        "sync_pulses",
        expect![[r#"Oc++rVrRSea7L5+dCz066kS/mPzhKZ8MhhvVo+8r5iY="#]],
    );

    // Check that stop actually stopped tape loading
    tester.emulator().stop_tape();
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_border(
        "stopped",
        expect![[r#"rIAW+jIqzRy5w0Xd+cqKAa6JVIgUhU5eTCvJ/kTmLuw="#]],
    );
}

#[test]
fn tape_rewind() {
    let mut settings = presets::settings_48k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("tape_rewind", settings);
    tester.load_tap("simple_tape.tap");
    // Play tape for some time while ROM loads
    tester.emulator().play_tape();
    tester.emulate_for(Duration::from_millis(4000));
    // Emulate LOAD ""
    tester.send_keystrokes(
        &[
            &[ZXKey::J],
            &[ZXKey::SymShift, ZXKey::P],
            &[ZXKey::SymShift, ZXKey::P],
            &[ZXKey::Enter],
        ],
        Duration::from_millis(100),
    );

    tester.emulator().rewind_tape().unwrap();
    tester.emulate_for(Duration::from_millis(8000));

    // Check tack tape is started loading after rewind
    tester.expect_screen(
        "loaded",
        expect![[r#"+o3MYnfBeDMtimIE/+6+o2/9h1OgtZ8izbO7b/jOiMc="#]],
    );
}

#[test]
fn fastload() {
    let mut tester = RustZXTester::new("fastload", presets::settings_48k_nosound());
    tester.load_tap("simple_tape.tap");
    tester.emulate_for(Duration::from_millis(45));
    tester.expect_screen(
        "running",
        expect![[r#"zoRX/GvcS0zqOJj3V0cmoZe56CNK2nXiJeH8pF8u1eg="#]],
    );
    tester.emulate_for(Duration::from_millis(10));
    tester.expect_screen(
        "finished",
        expect![[r#"zDQzdQr19uTYaZouk7ex+pkylk2TRFAuenooMVFjkyQ="#]],
    );
    tester.expect_border(
        "finished",
        expect![[r#"tmGY7e4h+XA3px6BcqnCXF83NEdBqVw8PW9sQtpMAvM="#]],
    );
}

#[test]
fn fastload_128k() {
    let mut tester = RustZXTester::new("fastload_128k", presets::settings_128k_nosound());
    tester.load_tap("simple_tape.tap");
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen(
        "loaded",
        expect![[r#"zDQzdQr19uTYaZouk7ex+pkylk2TRFAuenooMVFjkyQ="#]],
    );
    tester.expect_border(
        "loaded",
        expect![[r#"tmGY7e4h+XA3px6BcqnCXF83NEdBqVw8PW9sQtpMAvM="#]],
    );
}
