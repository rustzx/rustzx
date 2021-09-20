use rustzx_core::zx::keys::ZXKey;
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

#[test]
fn no_fastload() {
    let mut settings = presets::settings_48k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("no_fastload", settings);
    tester.load_tape("simple_tape.tap");
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
    tester.expect_screen("empty");
    tester.expect_border("empty");

    tester.emulator().play_tape();
    tester.emulate_for(Duration::from_millis(2000));

    // Check tack tape is started loading
    tester.expect_border("sync_pulses");

    // Check that data block started loading
    tester.emulate_for(Duration::from_millis(3100));
    tester.expect_border("data_pulses");

    // Check that Loader has been loaded
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen("block_1");

    // Check that second block has been loaded.
    tester.emulate_for(Duration::from_millis(45000));
    tester.expect_border("end");
    tester.expect_screen("block_2");
}

#[test]
fn no_fastload_128k() {
    let mut settings = presets::settings_128k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("no_fastload_128k", settings);
    tester.load_tape("simple_tape.tap");
    // Wait for ROM to load
    tester.emulate_for(Duration::from_millis(3000));

    // Emulate Enter keypress to load tape
    tester.send_keystrokes(&[&[ZXKey::Enter]], Duration::from_millis(100));

    // Check that tape is not loading until signaled manually
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen("ready_to_load");
    tester.expect_border("ready_to_load");

    tester.emulator().play_tape();
    tester.emulate_for(Duration::from_millis(2000));

    // Check tack tape is started loading
    tester.expect_border("sync_pulses");

    // Check that data block started loading
    tester.emulate_for(Duration::from_millis(3000));
    tester.expect_border("data_pulses");

    // Check that Loader has been loaded
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen("block_1");

    // Switching to the next block is already tested by `no_fastload` test, therefore
    // we can skip this for 128K test
}

#[test]
fn tape_stop() {
    let mut settings = presets::settings_48k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("tape_stop", settings);
    tester.load_tape("simple_tape.tap");
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
    tester.expect_border("sync_pulses");

    // Check that stop actually stopped tape loading
    tester.emulator().stop_tape();
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_border("stopped");
}

#[test]
fn tape_rewind() {
    let mut settings = presets::settings_48k_nosound();
    settings.tape_fastload_enabled = false;
    settings.autoload_enabled = false;

    let mut tester = RustZXTester::new("tape_rewind", settings);
    tester.load_tape("simple_tape.tap");
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
    tester.expect_screen("loaded");
}

#[test]
fn fastload() {
    let mut tester = RustZXTester::new("fastload", presets::settings_48k_nosound());
    tester.load_tape("simple_tape.tap");
    tester.emulate_for(Duration::from_millis(45));
    tester.expect_screen("running");
    tester.emulate_for(Duration::from_millis(10));
    tester.expect_screen("finished");
    tester.expect_border("finished");
}

#[test]
fn fastload_128k() {
    let mut tester = RustZXTester::new("fastload_128k", presets::settings_128k_nosound());
    tester.load_tape("simple_tape.tap");
    tester.emulate_for(Duration::from_millis(100));
    tester.expect_screen("loaded");
    tester.expect_border("loaded");
}
