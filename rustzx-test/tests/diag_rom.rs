use expect_test::expect;
use rustzx_core::zx::{keys::ZXKey, machine::ZXMachine};
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

const DIAG_ROM_NAME: &str = "diag_rom_v56.gz";
const DIAG_ROM_MENU_SNAP_128K_NAME: &str = "diag_rom_v56_started.128k.sna.gz";

fn load_diag_rom(t: &mut RustZXTester) {
    t.load_single_page_rom(DIAG_ROM_NAME);
}

fn load_diag_rom_menu_sna(t: &mut RustZXTester, machine: ZXMachine) {
    if machine != ZXMachine::Sinclair128K {
        panic!("Non-128K DiagROM snapshots are missing");
    }
    load_diag_rom(t);
    // NOTE: DiagROM is a little bit tricky about loading its state via snapshot, therefore
    // here we actually load SNA of emulator a few moments before showing actual DiagROM menu
    t.load_sna(DIAG_ROM_MENU_SNAP_128K_NAME);
    t.emulate_for(Duration::from_millis(1000));
}

#[test]
fn diag_rom_mem_48k() {
    let mut t = RustZXTester::new("diag_rom_mem_48k", presets::settings_48k_nosound());
    load_diag_rom(&mut t);
    t.emulate_for(Duration::from_secs(50));
    t.expect_screen(
        "result",
        expect![[r#"JYQl8vwFrIjaC+tZT34FBtey/aJHuylNvjldBTDIL0Q="#]],
    );
}

#[test]
fn diag_rom_mem_128k() {
    let mut t = RustZXTester::new("diag_rom_mem_128k", presets::settings_128k_nosound());
    load_diag_rom(&mut t);
    t.emulate_for(Duration::from_secs(50));
    t.expect_screen(
        "result",
        expect![[r#"JYQl8vwFrIjaC+tZT34FBtey/aJHuylNvjldBTDIL0Q="#]],
    );
}

#[test]
fn diag_rom_mem_banked_128k() {
    let mut t = RustZXTester::new("diag_rom_mem_banked_128k", presets::settings_128k_nosound());
    load_diag_rom_menu_sna(&mut t, ZXMachine::Sinclair128K);
    t.send_keystrokes(&[&[ZXKey::N7], &[ZXKey::N1]], Duration::from_millis(100));
    t.emulate_for(Duration::from_secs(100));
    t.expect_screen(
        "result",
        expect![[r#"27D2yD4yM6WJAv3vO3NyofTuSVfGOQXo8V0qrbc5wVk="#]],
    );
}

#[test]
fn diag_rom_second_screen_bank_128k() {
    let mut t = RustZXTester::new(
        "diag_rom_second_screen_bank_128k",
        presets::settings_128k_nosound(),
    );
    load_diag_rom_menu_sna(&mut t, ZXMachine::Sinclair128K);
    t.send_keystrokes(&[&[ZXKey::N7], &[ZXKey::N2]], Duration::from_millis(100));
    t.emulate_for(Duration::from_secs(3));
    t.expect_screen(
        "screen1",
        expect![[r#"+uu521vfpyy9IUSYYiqbiOu9s0NIV7gW8Co/ZXKRi2I="#]],
    );
    t.emulate_for(Duration::from_secs(4));
    t.expect_screen(
        "screen2",
        expect![[r#"ff2AqkCUYcPdp+yLMsOg8s3DyBpwC6YEvyUMR/RWPSs="#]],
    );
}

#[test]
fn diag_rom_contention_128k() {
    let mut t = RustZXTester::new("diag_rom_contention_128k", presets::settings_128k_nosound());
    load_diag_rom_menu_sna(&mut t, ZXMachine::Sinclair128K);
    t.send_keystrokes(&[&[ZXKey::N7], &[ZXKey::N5]], Duration::from_millis(100));
    t.emulate_for(Duration::from_secs(3));
    t.expect_screen(
        "result",
        expect![[r#"ImqjTr7XsJrZsDPYqB/wRgYachkgCHHPbCc+rH3Clg8="#]],
    );
}

#[test]
fn diag_rom_ula_128k() {
    let mut t = RustZXTester::new("diag_rom_ula_128k", presets::settings_128k_nosound());
    load_diag_rom_menu_sna(&mut t, ZXMachine::Sinclair128K);
    t.send_keystrokes(&[&[ZXKey::N6], &[ZXKey::N1]], Duration::from_millis(100));
    t.emulate_for(Duration::from_secs(3));
    t.expect_screen(
        "result",
        expect![[r#"I2YQuImYmwfzNB9Y48M4ce2JVwDsQcEO2jamkLoBYxs="#]],
    );
}

#[test]
fn diag_rom_interrupt_128k() {
    let mut t = RustZXTester::new("diag_rom_interrupt_128k", presets::settings_128k_nosound());
    load_diag_rom_menu_sna(&mut t, ZXMachine::Sinclair128K);
    t.send_keystrokes(&[&[ZXKey::N6], &[ZXKey::N4]], Duration::from_millis(100));
    t.emulate_for(Duration::from_secs(2));
    t.expect_screen(
        "frame1",
        expect![[r#"47E1ralzgtRVSmo6f1e9bMHyw1S+b4F/zQOTVo0bfq0="#]],
    );
    t.emulate_for(Duration::from_secs(2));
    t.expect_screen(
        "frame2",
        expect![[r#"ZmdepGrZWq8SvP+GeXziGeuNj0T0nR0VL6XGXPN6DAw="#]],
    );
}
