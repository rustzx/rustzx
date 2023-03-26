// use expect_test::expect;
use rustzx_test::framework::{presets, RustZXTester};
use std::time::Duration;

const PROG_START: u16 = 0x8000;

/// We are tryint to find call opcode of the following `z80test` code (main.asm):
/// ```text
/// 65 | .ok    call    print                       ; print success message
/// 66 |        db      "all tests passed.",13,0
/// ```
fn search_success_print_call(t: &mut RustZXTester) -> u16 {
    // Search in first 1k of program
    const MAX_SEARCH_OFFSET: u16 = 1024;
    const SEARCH_PATTERN: &[u8] = b"all tests passed";
    const CALL_OPCODE_LEN: u16 = 3;

    for mem_offset in 0..MAX_SEARCH_OFFSET {
        let mut found = true;
        for (pattern_offset, expected) in SEARCH_PATTERN.into_iter().copied().enumerate() {
            if t.peek(PROG_START + mem_offset + pattern_offset as u16) != expected {
                found = false;
                break;
            }
        }
        if found {
            return PROG_START + mem_offset - CALL_OPCODE_LEN;
        }
    }

    panic!("Failed to found success print call in loaded program");
}

fn run_z80_test(name: &str) {
    const TIMEOUT_INIT: Duration = Duration::from_secs(1);
    const TIMEOUT_TEST: Duration = Duration::from_secs(350);

    let mut tester = RustZXTester::new(name, presets::settings_48k_nosound());
    tester.disable_scroll_message();
    tester.load_tap(format!("{}.tap.gz", name));

    // Wait until emulator loads program and jumps to PROG_START
    tester.emulate_until_breakpoint(PROG_START, TIMEOUT_INIT);
    let success_breakpoint_addr = search_success_print_call(&mut tester);

    // Wait test to succeed
    tester.emulate_until_breakpoint(success_breakpoint_addr, TIMEOUT_TEST);
}

#[test]
#[ignore]
fn z80full() {
    run_z80_test("z80full");
}

#[test]
#[ignore]
fn z80ccf() {
    run_z80_test("z80ccf");
}

#[test]
#[ignore]
fn z80memptr() {
    run_z80_test("z80memptr");
}
