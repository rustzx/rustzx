use std::io::Read;
use std::fs::File;

use time;
use z80::*;
use z80::opcodes::execute_pop_16;
use zx::{ZXController, ZXMachine};
use zx::colors::ZXColor;
use utils::*;

pub struct Emulator {
    machine: ZXMachine,
    cpu: Z80,
    pub controller: ZXController,
    speed: EmulationSpeed,
    fast_load: bool,
    sound_enabled: bool,
}

impl Emulator {
    pub fn new(machine: ZXMachine) -> Emulator {
        Emulator {
            machine: machine,
            cpu: Z80::new(),
            controller: ZXController::new(machine),
            speed: EmulationSpeed::Definite(1),
            fast_load: false,
            sound_enabled: true,
        }
    }

    /// changes emulation speed
    pub fn set_speed(&mut self, new_speed: EmulationSpeed) {
        self.speed = new_speed;
    }

    /// changes fast loading flag
    pub fn set_fast_load(&mut self, value: bool) {
        self.fast_load = value;
    }

    /// changes sound playback flag
    pub fn set_sound(&mut self, value: bool) {
        self.sound_enabled = value;
    }

    /// function for sound generation request check
    pub fn have_sound(&self) -> bool {
        if let EmulationSpeed::Definite(1) = self.speed {
            if self.sound_enabled {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// SNA snapshot loading function
    pub fn load_sna(&mut self, file: &str) {
        let mut data = Vec::new();
        File::open(file).unwrap().read_to_end(&mut data).unwrap();
        assert!(data.len() == 49179);
        // i-reg
        self.cpu.regs.set_i(data[0]);
        // alt-regs
        self.cpu.regs.set_hl(make_word(data[2], data[1]));
        self.cpu.regs.set_de(make_word(data[4], data[3]));
        self.cpu.regs.set_bc(make_word(data[6], data[5]));
        self.cpu.regs.exx();
        // af'
        self.cpu.regs.set_af(make_word(data[8], data[7]));
        self.cpu.regs.swap_af_alt();
        // regs
        self.cpu.regs.set_hl(make_word(data[10], data[9]));
        self.cpu.regs.set_de(make_word(data[12], data[11]));
        self.cpu.regs.set_bc(make_word(data[14], data[13]));
        // index regs
        self.cpu.regs.set_iy(make_word(data[16], data[15]));
        self.cpu.regs.set_ix(make_word(data[18], data[17]));
        // iff1, iff2
        self.cpu.regs.set_iff1((data[19] & 0x01) != 0);
        self.cpu.regs.set_iff1((data[19] & 0x04) != 0);
        // r
        self.cpu.regs.set_r(data[20]);
        // af
        self.cpu.regs.set_af(make_word(data[22], data[21]));
        // sp
        self.cpu.regs.set_sp(make_word(data[24], data[23]));
        // interrupt mode
        self.cpu.set_im(data[25]);
        // set border
        self.controller.border.set_border(Clocks(0), ZXColor::from_bits(data[26]));
        // ram pages
        self.controller.memory.load_ram(0, &data[27..16411]);
        // validate screen, it has been changed
        self.controller.validate_screen();
        self.controller.memory.load_ram(1, &data[16411..32795]);
        self.controller.memory.load_ram(2, &data[32795..49179]);
        // RET
        execute_pop_16(&mut self.cpu,
                       &mut self.controller,
                       RegName16::PC,
                       Clocks(0));
    }

    /// events processing function
    fn process_event(&mut self, event: Event) {
        let Event { kind: e, time: _ } = event;
        match e {
            // NOTE: This can be moved to new mod, `loaders` or something
            // Fast tape loading found, use it
            EventKind::FastTapeLoad if self.controller.tape.can_fast_load() && self.fast_load => {
                // resetting tape pos to beginning.
                self.controller.tape.reset_pos_in_block();
                // So, at current moment we at 0x056C in 48K Rom.
                // AF contains some garbage. so we need to swap if wtih A'F'
                self.cpu.regs.swap_af_alt();
                // now we have type of block at A and flags before LD-BYTES at F
                let mut f = self.cpu.regs.get_flags();
                let mut acc = self.cpu.regs.get_acc();
                // variable to store resulting flags
                let mut result_flags;
                // pos relative to block start
                let mut pos = 0;
                // destination address in RAM
                let mut dest = self.cpu.regs.get_reg_16(RegName16::IX);
                // remaining length
                let mut length = self.cpu.regs.get_reg_16(RegName16::DE);
                // parity accumulator and current byte (h, l) regs
                let (mut parity_acc, mut current_byte) = (0, 0);
                'loader: loop {
                    // if we still on block
                    if let Some(byte) = self.controller.tape.block_byte(pos) {
                        // set current byte, shift position and do parity check iteration
                        current_byte = byte;
                        pos += 1;
                        parity_acc ^= current_byte;
                        // no bytes left, set A to parity accumulator (works as in ROM)
                        // and check parity last time
                        if length == 0 {
                            acc = parity_acc;
                            // consider we CAN have parity error
                            result_flags = Some(0);
                            // if checksum correct set carry to prevent error
                            if acc == 0 {
                                result_flags = Some(FLAG_CARRY);
                            }
                            break 'loader;
                        }
                        // block type check, first byte
                        if (f & FLAG_ZERO) == 0 {
                            acc ^= current_byte;
                            // if type wrong
                            if acc != 0 {
                                result_flags = Some(0);
                                break 'loader;
                            }
                            // type check passed, go to next byte;
                            f |= FLAG_ZERO;
                            continue;
                        }
                        // LOAD
                        if (f & FLAG_CARRY) != 0 {
                            self.controller.write_internal(dest, current_byte);
                            // VERIFY
                        } else {
                            // check for parity each byte, if this fails - set flags to error state
                            acc = self.controller.memory.read(dest) ^ current_byte;
                            if acc != 0 {
                                result_flags = Some(0);
                                break 'loader;
                            }
                        }
                        // move destination pointer and decrease count of remaining bytes
                        dest += 1;
                        length -= 1;
                    } else {
                        // this happens if requested length and provided are not matched
                        result_flags = Some(FLAG_ZERO);
                        break 'loader;
                    }
                }
                // set regs to new state
                self.cpu.regs.set_reg_16(RegName16::IX, dest);
                self.cpu.regs.set_reg_16(RegName16::DE, length);
                self.cpu.regs.set_hl(make_word(parity_acc, current_byte));
                self.cpu.regs.set_acc(acc);
                // set new flag, if something changed
                if let Some(new_flags) = result_flags {
                    f = new_flags;
                    // RET
                    opcodes::execute_pop_16(&mut self.cpu,
                                            &mut self.controller,
                                            RegName16::PC,
                                            Clocks(0));
                }
                self.cpu.regs.set_flags(f);
                // move to next block
                self.controller.tape.next_block();
            }
            _ => {}
        }
    }

    // processes all events, happened at frame emulation cycle
    fn process_all_events(&mut self) {
        loop {
            if let Some(event) = self.controller.pop_event() {
                self.process_event(event);
            } else {
                break;
            }
        }
    }

    /// Emulate frames, maximum in `max_time` time, returns emulation time in nanoseconds
    /// in most cases time is max 1/50 of second, even when using
    /// loader acceleration
    pub fn emulate_frame(&mut self, max_time: u64) -> u64 {
        let mut time = 0u64;
        'frame: loop {
            // start of current frame
            let start_time = time::precise_time_ns();
            // reset controller internal frame counter
            self.controller.reset_frame_counter();
            'cpu: loop {
                // Emulation step. if instant event happened then accept in and execute
                if !self.cpu.emulate(&mut self.controller) {
                    if let Some(event) = self.controller.pop_event() {
                        self.process_event(event);
                    }
                }
                // If speed is defined
                if let EmulationSpeed::Definite(multiplier) = self.speed {
                    if self.controller.frames_count() >= multiplier {
                        // no more frames
                        self.controller.clear_events();
                        return time::precise_time_ns() - start_time;
                    };
                    // if speed is maximal.
                } else {
                    // if any frame passed then break cpu loop, but try to start new frame
                    if self.controller.frames_count() != 0 {
                        break 'cpu;
                    }
                }
            }
            time += time::precise_time_ns() - start_time;
            // if time is bigger than `max_time` then stop emulation cycle
            if time > max_time {
                break 'frame;
            }
        }
        self.controller.clear_events();
        return time;
    }
}
