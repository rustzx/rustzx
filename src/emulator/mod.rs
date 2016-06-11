use time;
use z80::*;
use zx::{ZXController, ZXMachine};
use utils::*;

pub struct Emulator {
    machine: ZXMachine,
    cpu: Z80,
    pub controller: ZXController,
    speed: EmulationSpeed,
}

impl Emulator {
    pub fn new(machine: ZXMachine) -> Emulator {
        Emulator {
            machine: machine,
            cpu: Z80::new(),
            controller: ZXController::new(machine),
            speed: EmulationSpeed::Definite(1),
        }
    }

    pub fn set_speed(&mut self, new_speed: EmulationSpeed) {
        self.speed = new_speed;
    }

    fn process_event(&mut self, event: Event) {
        let Event { kind: e, time: _ } = event;
        match e {
            // NOTE: This can be moved to new mod, `loaders` or something
            // Fast tape loading found, use it
            EventKind::FastTapeLoad if self.controller.tape.can_fast_load() => {
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
                            self.controller.memory.write(dest, current_byte);
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
            };
            time += time::precise_time_ns() - start_time;
            // if time is bigger than `max_time` then stop emulation cycle
            if time > max_time {
                break 'frame;
            }
        };
        self.controller.clear_events();
        return time;
    }
}
