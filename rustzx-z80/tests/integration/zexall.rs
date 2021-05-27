// Testing strategy was inspired by https://github.com/anotherlin/z80emu
const ZEXALL_IMAGE_BASE_ADDRESS: u16 = 0x0100;
const ZEXALL_IMAGE: &[u8] = include_bytes!("assets/zexall.com");
const ZEXALL_TEST_POINTERS_TABLE_ADDRESS: u16 = 0x013A;
const ZEXALL_BEGIN_MESSAGE_ADDRESS: u16 = 0x01DDA;
const ZEXALL_END_MESSAGE_ADDRESS: u16 = 0x01DF6;
const ZEXALL_INITIAL_SP_VALUE_ADDRESS: u16 = 0x0006;
const ZEXALL_RESET_ADDRESS: u16 = 0x0000;
const ZEXALL_SUCCESS_MESSAGE_SUFFIX: &str = "  OK\n\r";

const BDOS_STRING_TERMINATOR: u8 = b'$';
const BDOS_SYSCALL_ADDRESS: u16 = 0x0005;
const BDOS_MAX_STRING_SIZE: u16 = 100;
const BDOS_SYSCALL_PRINT_CHAR: u8 = 2;
const BDOS_SYSCALL_PRINT_STRING: u8 = 9;

const MEMORY_SIZE: usize = 64 * 1024;

const RET_OPCODE: u8 = 0xc9;

use crate::TestingBus;
use core::panic;
use rustzx_z80::Z80;

struct ZexallTester {
    cpu: Z80,
    bus: TestingBus,
}

impl ZexallTester {
    fn new(test_id: u16) -> Self {
        let mut cpu = Z80::default();
        cpu.regs.set_pc(ZEXALL_IMAGE_BASE_ADDRESS);

        let mut bus = TestingBus::new(MEMORY_SIZE);
        bus.load_to_memory(ZEXALL_IMAGE, ZEXALL_IMAGE_BASE_ADDRESS);

        // Silence redundant messages
        bus.patch_memory(ZEXALL_BEGIN_MESSAGE_ADDRESS, BDOS_STRING_TERMINATOR);
        bus.patch_memory(ZEXALL_END_MESSAGE_ADDRESS, BDOS_STRING_TERMINATOR);

        // Immediate return after "syscall". Actual logic will be performed
        // when breakpoint on this address will be detected
        bus.patch_memory(BDOS_SYSCALL_ADDRESS, RET_OPCODE);

        let [sp_l, sp_h] = 0xC000u16.to_le_bytes();
        bus.patch_memory(ZEXALL_INITIAL_SP_VALUE_ADDRESS, sp_l);
        bus.patch_memory(ZEXALL_INITIAL_SP_VALUE_ADDRESS + 1, sp_h);

        // patch tests table to execute only one test
        let test_pointer_addr = ZEXALL_TEST_POINTERS_TABLE_ADDRESS + test_id * 2;
        let test_pointer_l = bus.read_memory(test_pointer_addr);
        let test_pointer_h = bus.read_memory(test_pointer_addr + 1);
        for (idx, byte) in [test_pointer_l, test_pointer_h, 0, 0].iter().enumerate() {
            bus.patch_memory(ZEXALL_TEST_POINTERS_TABLE_ADDRESS + idx as u16, *byte);
        }

        bus.add_breakpoint(BDOS_SYSCALL_ADDRESS);
        bus.add_breakpoint(ZEXALL_RESET_ADDRESS);

        Self { cpu, bus }
    }

    fn execute(&mut self) {
        let mut output = String::new();
        loop {
            let syscall_id = self.cpu.regs.get_c();

            match self.bus.last_breakpoint() {
                Some(ZEXALL_RESET_ADDRESS) => {
                    break;
                }
                Some(BDOS_SYSCALL_ADDRESS) if syscall_id == BDOS_SYSCALL_PRINT_CHAR => {
                    output.push(self.cpu.regs.get_e() as char);
                }
                Some(BDOS_SYSCALL_ADDRESS) if syscall_id == BDOS_SYSCALL_PRINT_STRING => {
                    let str_base = self.cpu.regs.get_de();
                    let mut offset = 0;
                    loop {
                        match self.bus.read_memory(str_base + offset) as char {
                            '$' => break,
                            ch => output.push(ch),
                        };
                        offset += 1;
                        if offset > BDOS_MAX_STRING_SIZE {
                            panic!("Too long zexall string");
                        }
                    }
                }
                _ => {}
            }

            self.cpu.emulate(&mut self.bus);
        }

        if !output.ends_with(ZEXALL_SUCCESS_MESSAGE_SUFFIX) {
            panic!("ERROR. Test output: {}", output);
        }
    }
}

macro_rules! zexall_tests_internal {
    [$idx:expr] => {};
    [$idx:expr, $name:ident] => {
        paste::paste! {
            #[ignore]
            #[test]
            pub fn [<zexall_ $name>]() {
                ZexallTester::new($idx).execute();
            }
        }
    };
    [$idx:expr, $name_head:ident, $($name_tail:ident),*] => {
        zexall_tests_internal![$idx, $name_head];
        zexall_tests_internal![$idx + 1, $($name_tail),*];
    };
}

macro_rules! zexall_tests {
    [$($name:ident),+ $(,)?] => {
        zexall_tests_internal![0, $($name),+];
    };
}

zexall_tests![
    adc16, add16, add16x, add16y, alu8i, alu8r, alu8rx, alu8x, bitx, bitz80, cpd1, cpi1, daa, inca,
    incb, incbc, incc, incd, incde, ince, inch, inchl, incix, inciy, incl, incm, incsp, incx,
    incxh, incxl, incyh, incyl, ld161, ld162, ld163, ld164, ld165, ld166, ld167, ld168, ld16im,
    ld16ix, ld8bd, ld8im, ld8imx, ld8ix1, ld8ix2, ld8ix3, ld8ixy, ld8rr, ld8rrx, lda, ldd1, ldd2,
    ldi1, ldi2, neg, rld, rot8080, rotxy, rotz80, srz80, srzx, st8ix1, st8ix2, st8ix3, stabd,
];
