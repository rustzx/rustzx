use expect_test::expect;
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

/// Address where emulator loads program from tape
const PROG_START: u16 = 0x8000;

/// Finds the address of pattern in emulator memory via peek method.
fn search_pattern(
    t: &mut RustZXTester,
    pattern: &[u8],
    min_search_range: u16,
    max_search_range: u16,
) -> u16 {
    assert!(
        min_search_range <= max_search_range,
        "min_search_range must be less or equal to max_search_range"
    );

    assert!(
        pattern.len() <= (max_search_range - min_search_range) as usize,
        "Pattern is too big"
    );

    for mem_offset in min_search_range..max_search_range {
        let mut found = true;
        for (pattern_offset, expected) in pattern.into_iter().copied().enumerate() {
            if t.peek(mem_offset + pattern_offset as u16) != expected {
                found = false;
                break;
            }
        }
        if found {
            return mem_offset;
        }
    }

    panic!("Failed to found pattern in loaded program");
}

/// We are trying to find call opcode of the following `z80test` code (main.asm):
/// ```text
/// 65 | .ok    call    print                       ; print success message
/// 66 |        db      "all tests passed.",13,0
/// ```
fn search_success_print_call(t: &mut RustZXTester) -> u16 {
    // Search in first 1k of program
    const MAX_SEARCH_BYTES: u16 = 1024;
    const SEARCH_PATTERN: &[u8] = b"all tests passed";
    const CALL_OPCODE_LEN: u16 = 3;

    let pattern_offset =
        search_pattern(t, SEARCH_PATTERN, PROG_START, PROG_START + MAX_SEARCH_BYTES);

    assert!(
        pattern_offset >= CALL_OPCODE_LEN,
        "Pattern offset is too small"
    );

    pattern_offset - CALL_OPCODE_LEN
}

/// Runs tests based on z80test project tape files.
fn run_z80_test(name: &str) {
    const TIMEOUT_INIT: Duration = Duration::from_secs(1);
    const TIMEOUT_TEST: Duration = Duration::from_secs(350);

    let mut t = RustZXTester::new(name, presets::settings_48k_nosound());
    // Disable scroll message to eliminate need for enter key press emulation during test run
    t.disable_scroll_message();
    t.load_tap(format!("{}.tap.gz", name));

    // Wait until emulator loads program and jumps to PROG_START
    t.emulate_until_breakpoint(PROG_START, TIMEOUT_INIT);
    let success_breakpoint_addr = search_success_print_call(&mut t);

    // Wait test to succeed
    t.emulate_until_breakpoint(success_breakpoint_addr, TIMEOUT_TEST);
}

macro_rules! z80test {
    ($($name:ident),*) => {
        $(
            #[test]
            #[ignore]
            fn $name() {
                run_z80_test(stringify!($name));
            }
        )*
    };
}

z80test!(z80full, z80ccf, z80memptr);

/// This code finds the following opcodes addresses
/// ```text
/// 294 | .exit: di             ; F3
/// 295 | .sp+1: ld    sp,0     ; 0x31 0x00  0x00
/// ```
fn search_exit_address(t: &mut RustZXTester) -> u16 {
    // Opcodes should be around 0x200 offset, lets search for 0x400 bytes
    const SEARCH_BYTES: u16 = 1024;
    const SEARCH_PATTERN: &[u8] = &[0xF3, 0x31, 0x00, 0x00];
    search_pattern(t, SEARCH_PATTERN, PROG_START, PROG_START + SEARCH_BYTES)
}

/// Tests z80 block instructions flags via z80bltst.tap from ZXSpectrumNextTests project
#[test]
fn z80_block_instruction_flags() {
    const TIMEOUT_INIT: Duration = Duration::from_secs(1);
    const TIMEOUT_TEST: Duration = Duration::from_secs(30);

    let mut t = RustZXTester::new(
        "z80_block_instruction_flags",
        presets::settings_48k_nosound(),
    );
    t.load_tap("z80bltst.tap.gz");

    // Wait until emulator loads program and jumps to PROG_START
    t.emulate_until_breakpoint(PROG_START, TIMEOUT_INIT);
    let exit_breakpoint_addr = search_exit_address(&mut t);

    // Wait test to succeed
    t.emulate_until_breakpoint(exit_breakpoint_addr, TIMEOUT_TEST);

    // Prepare screen to be compared against expected frame hash
    t.emulate_frame();

    t.expect_screen(
        "frame1",
        expect![[r#"KnD/3IYvk72FtwHyxhia4wNWGwcJKgmnxZfTmaebTlo="#]],
    );
}
