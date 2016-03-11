use super::*;
use z80::*;
use utils::*;

/// normal execution group, can be modified with prefixes DD, FD, providing
/// DD OPCODE [NN], FD OPCODE [NN] instruction group
///
/// Opcode matching organised based on
/// [document](http://www.z80.info/decoding.htm) by Cristian Dinu
///
/// DAA algorithm
/// [link](http://www.worldofspectrum.org/faq/reference/z80reference.htm#DAA)
pub fn execute_normal(cpu: &mut Z80, bus: &mut Z80Bus, opcode: Opcode, prefix: Prefix) -> Clocks {
    let mut clocks = 0;
    // 2 first bits of opcode
    match opcode.x {
        // ---------------------------------
        // [0b00yyyzzz] instruction section
        // ---------------------------------
        // [0b00yyy000] instruction group (NOP, EX, DJNZ, JR)
        U2::N0 if opcode.z == U3::N0 => {
            match opcode.y {
                // NOP
                // [0b00000000] = 0x00
                U3::N0 => {}
                // EX AF, AF'
                // [0b00001000] = 0x08
                U3::N1 => {
                    cpu.regs.swap_af_alt();
                }
                // DJNZ offset;   13/8 clocks
                // [0b00010000] = 0x10
                U3::N2 => {
                    let offset = cpu.rom_next_byte(bus) as i8;
                    // preform jump
                    if cpu.regs.dec_reg_8(RegName8::B, 1) != 0 {
                        cpu.regs.shift_pc(offset);
                        clocks += 13;
                    } else {
                        clocks += 8;
                    };
                    // pc already pointing to next instruction
                }
                // JR offset
                // [0b00011000] = 0x18
                U3::N3 => {
                    let offset = cpu.rom_next_byte(bus) as i8;
                    cpu.regs.shift_pc(offset);
                }
                // JR condition[y-4] displacement;
                // NZ [0b00100000], Z [0b00101000] NC [0b00110000] C [0b00111000]
                U3::N4 | U3::N5 | U3::N6 | U3::N7 => {
                    // 0x20, 0x28, 0x30, 0x38
                    let offset = cpu.rom_next_byte(bus) as i8;
                    // y in range 4..7, non-wrapped sub allowed
                    let cnd = Condition::from_u3(U3::from_byte(opcode.y.as_byte() - 4, 0));
                    if cpu.regs.eval_condition(cnd) {
                        cpu.regs.shift_pc(offset);
                        clocks += 12;
                    } else {
                        clocks += 7;
                    };
                }
            };
        }
        // [0b00ppq001] instruction group (LD, ADD)
        U2::N0 if opcode.z == U3::N1 => {
            match opcode.q {
                // LD rp[p], nn
                // [0b00pp0001] : 0x01, 0x11, 0x21, 0x31
                U1::N0 => {
                    let reg = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
                    let data = cpu.rom_next_word(bus);
                    cpu.regs.set_reg_16(reg, data);
                }
                // ADD HL/IX/IY, ss ; ss - 16 bit with sp set
                // [0b00pp1001] : 0x09; 0x19; 0x29; 0x39
                U1::N1 => {
                    let reg_operand = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
                    let reg_acc = RegName16::HL.with_prefix(prefix);
                    let acc = cpu.regs.get_reg_16(reg_acc);
                    let operand = cpu.regs.get_reg_16(reg_operand);
                    // calc half_carry
                    let half_carry = half_carry_16(acc, operand);
                    let (acc, carry) = acc.overflowing_add(operand);
                    // check flags!
                    cpu.regs.set_flag(Flag::Carry, carry); //set carry
                    cpu.regs.set_flag(Flag::Sub, false); // is addition
                    cpu.regs.set_flag(Flag::HalfCarry, half_carry); // half carry
                    cpu.regs.set_flag(Flag::F3, acc & 0x0800 != 0); // 3 bit of hi
                    cpu.regs.set_flag(Flag::F5, acc & 0x2000 != 0); // 5 bit of hi
                    // set register!
                    cpu.regs.set_reg_16(reg_acc, acc);
                }
            };
        }
        // [0b00ppq010] instruction group (LD INDIRECT)
        U2::N0 if opcode.z == U3::N2 => {
            match opcode.q {
                // LD (BC), A
                // [0b00000010] : 0x02
                U1::N0 if opcode.p == U2::N0 => {
                    bus.write(cpu.regs.get_reg_16(RegName16::BC),
                              cpu.regs.get_reg_8(RegName8::A));
                }
                // LD (DE), A
                // [0b00010010] : 0x12
                U1::N0 if opcode.p == U2::N1 => {
                    bus.write(cpu.regs.get_reg_16(RegName16::DE),
                              cpu.regs.get_reg_8(RegName8::A));
                }
                // LD (nn), HL/IX/IY
                // [0b00100010] : 0x22
                U1::N0 if opcode.p == U2::N2 => {
                    let addr = cpu.rom_next_word(bus);
                    let reg = RegName16::HL.with_prefix(prefix);
                    bus.write_word(addr, cpu.regs.get_reg_16(reg));
                }
                // LD (nn), A
                // [0b00110010] : 0x32
                U1::N0 => {
                    let addr = cpu.rom_next_word(bus);
                    bus.write(addr, cpu.regs.get_reg_8(RegName8::A));
                }
                // LD A, (BC)
                // [0b00001010] : 0x0A
                U1::N1 if opcode.p == U2::N0 => {
                    let addr = cpu.regs.get_reg_16(RegName16::BC);
                    cpu.regs.set_reg_8(RegName8::A, bus.read(addr));
                }
                // LD A, (DE)
                // [0b00011010] : 0x1A
                U1::N1 if opcode.p == U2::N1 => {
                    let addr = cpu.regs.get_reg_16(RegName16::BC);
                    cpu.regs.set_reg_8(RegName8::A, bus.read(addr));
                }
                // LD HL/IX/IY, (nn)
                // [0b00101010] : 0x2A
                U1::N1 if opcode.p == U2::N2 => {
                    let addr = cpu.rom_next_word(bus);
                    let reg = RegName16::HL.with_prefix(prefix);
                    cpu.regs.set_reg_16(reg, bus.read_word(addr));
                }
                // LD A, (nn)
                // [0b00111010] : 0x3A
                U1::N1 => {
                    let addr = cpu.rom_next_word(bus);
                    cpu.regs.set_reg_8(RegName8::A, bus.read(addr));
                }
            };
        }
        // [0b00ppq011] instruction group (INC, DEC)
        U2::N0 if opcode.z == U3::N3 => {
            // get register by rp[pp]
            let reg = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
            match opcode.q {
                // INC BC/DE/HL/IX/IY/SP
                // [0b00pp0011] : 0x03, 0x13, 0x23, 0x33
                U1::N0 => {
                    cpu.regs.inc_reg_16(reg, 1);
                }
                // DEC BC/DE/HL/IX/IY/SP
                // [0b00pp1011] : 0x03, 0x13, 0x23, 0x33
                U1::N1 => {
                    cpu.regs.dec_reg_16(reg, 1);
                }
            };
        }
        // [0b00yyy100], [0b00yyy101] instruction group (INC, DEC) 8 bit
        U2::N0 if (opcode.z == U3::N4) || (opcode.z == U3::N5) => {
            let operand;
            let data;
            let result;
            // ------------
            //   get data
            // ------------
            if let Some(mut reg) = RegName8::from_u3(opcode.y) {
                // INC r[y], DEC y[y] ; IX and IY also used
                // INC [0b00yyy100] : 0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x3C
                // DEC [0b00yyy101] : 0x05, 0x0D, 0x15, 0x1D, 0x25, 0x2D, 0x3D
                reg = reg.with_prefix(prefix);
                data = cpu.regs.get_reg_8(reg);
                operand = LoadOperand8::Reg(reg);
            } else {
                // INC (HL)/(IX + d)/(IY + d), DEC (HL)/(IX + d)/(IY + d) ; INDIRECT
                // INC [0b00110100], DEC [0b00110101] : 0x34, 0x35
                let addr = if prefix == Prefix::None {
                    // we have IND/DEC (HL)
                    cpu.regs.get_reg_16(RegName16::HL)
                } else {
                    // we have INC/DEC (IX/IY + d)
                    let d = cpu.rom_next_byte(bus) as i8;
                    word_displacement(cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)),d)
                };
                // read data
                data = bus.read(addr);
                operand = LoadOperand8::Indirect(addr);
            };
            // ------------
            //   execute
            // ------------
            if opcode.z == U3::N4 {
                // INC
                result = data.wrapping_add(1);
                cpu.regs.set_flag(Flag::Sub, false);
                cpu.regs.set_flag(Flag::ParityOveflow, data == 0x7F);
                cpu.regs.set_flag(Flag::HalfCarry, half_carry_8(data, 1));
            } else {
                // DEC
                result = data.wrapping_sub(1);
                cpu.regs.set_flag(Flag::Sub, true);
                cpu.regs.set_flag(Flag::ParityOveflow, data == 0x80);
                cpu.regs.set_flag(Flag::HalfCarry, half_borrow_8(data, 1));
            }
            cpu.regs.set_flag(Flag::Zero, result == 0);
            cpu.regs.set_flag(Flag::Sign, result & 0x80 != 0); // last bit check
            cpu.regs.set_flag(Flag::F3, result & 0x08 != 0); // 3 bit
            cpu.regs.set_flag(Flag::F5, result & 0x20 != 0); // 5 bit
            // ------------
            //  write data
            // ------------
            match operand {
                LoadOperand8::Indirect(addr) => {
                    bus.write(addr, result);
                }
                LoadOperand8::Reg(reg) => {
                    cpu.regs.set_reg_8(reg, result);
                }
            };
        }
        // [0b00yyy110] instruction group (LD R, N 8 bit) :
        // 0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E
        U2::N0 if opcode.z == U3::N6 => {
            let operand = if let Some(reg) = RegName8::from_u3(opcode.y) {
                // Direct LD R, N
                LoadOperand8::Reg(reg.with_prefix(prefix))
            } else {
                // INDIRECT LD (HL/IX+d/IY+d), N <PREFIX>[0b00110110] : 0x36
                if prefix == Prefix::None {
                    // LD (HL)
                    LoadOperand8::Indirect(cpu.regs.get_reg_16(RegName16::HL))
                } else {
                    // LD (IX+d/ IY+d)
                    let d = cpu.rom_next_byte(bus) as i8;
                    LoadOperand8::Indirect(word_displacement(
                        cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)),d))
                }
            };
            // Read const operand
            let data = cpu.rom_next_byte(bus);
            // write to bus or reg
            match operand {
                LoadOperand8::Indirect(addr) => {
                    bus.write(addr, data);
                }
                LoadOperand8::Reg(reg) => {
                    cpu.regs.set_reg_8(reg, data);
                }
            };
        }
        // [0b00yyy111] instruction group (Assorted)
        U2::N0 => {
            match opcode.y {
                // RLCA ; Rotate left; msb will become lsb; carry = msb
                // [0b00000111] : 0x07
                U3::N0 => {
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x80) != 0;
                    data = data.wrapping_shl(1);
                    if carry {
                        data |= 1;
                    } else {
                        data &= 0xFE;
                    };
                    cpu.regs.set_flag(Flag::HalfCarry, false);
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::Carry, carry);
                    cpu.regs.set_flag(Flag::F3, data & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, data & 0x20 != 0); // 5 bit
                    cpu.regs.set_acc(data);
                }
                // RRCA ; Rotate right; lsb will become msb; carry = lsb
                // [0b00001111] : 0x0F
                U3::N1 => {
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x01) != 0;
                    data = data.wrapping_shr(1);
                    if carry {
                        data |= 0x80;
                    } else {
                        data &= 0x7F;
                    };
                    cpu.regs.set_flag(Flag::HalfCarry, false);
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::Carry, carry);
                    cpu.regs.set_flag(Flag::F3, data & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, data & 0x20 != 0); // 5 bit
                    cpu.regs.set_acc(data);
                }
                // RLA Rotate left trough carry
                // [0b00010111]: 0x17
                U3::N2 => {
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x80) != 0;
                    data = data.wrapping_shl(1);
                    if cpu.regs.get_flag(Flag::Carry) {
                        data |= 1;
                    } else {
                        data &= 0xFE;
                    };
                    cpu.regs.set_flag(Flag::HalfCarry, false);
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::Carry, carry);
                    cpu.regs.set_flag(Flag::F3, data & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, data & 0x20 != 0); // 5 bit
                    cpu.regs.set_acc(data);
                }
                // RRA Rotate right trough carry
                // [0b00011111] : 0x1F
                U3::N3 => {
                    let before = cpu.regs.get_acc();
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x01) != 0;
                    data = data.wrapping_shr(1);
                    if cpu.regs.get_flag(Flag::Carry) {
                        data |= 0x80;
                    } else {
                        data &= 0x7F;
                    };
                    cpu.regs.set_flag(Flag::HalfCarry, false);
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::Carry, carry);
                    cpu.regs.set_flag(Flag::F3, data & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, data & 0x20 != 0); // 5 bit
                    cpu.regs.set_acc(data);
                }
                // DAA [0b00100111] [link to the algorithm in header]
                U3::N4 => {
                    let acc = cpu.regs.get_acc();
                    let mut correction;
                    if (acc > 0x99) || cpu.regs.get_flag(Flag::Carry) {
                        correction = 0x60_u8;
                        cpu.regs.set_flag(Flag::Carry, true);
                    } else {
                        correction = 0x00_u8;
                        cpu.regs.set_flag(Flag::Carry, false);
                    };
                    if ((acc & 0x0F) > 0x09) || cpu.regs.get_flag(Flag::HalfCarry) {
                        correction |= 0x06;
                    }
                    let acc_new = if !cpu.regs.get_flag(Flag::Sub) {
                        cpu.regs.set_flag(Flag::HalfCarry, half_carry_8(acc, correction));
                        acc.wrapping_add(correction)
                    } else {
                        cpu.regs.set_flag(Flag::HalfCarry, half_borrow_8(acc, correction));
                        acc.wrapping_sub(correction)
                    };
                    cpu.regs.set_flag(Flag::Sign, acc_new & 0x80 != 0); // Sign
                    cpu.regs.set_flag(Flag::Zero, acc_new == 0); // Zero
                    cpu.regs.set_flag(Flag::ParityOveflow,
                                       tables::PARITY_BIT[acc_new as usize] != 0);
                    cpu.regs.set_flag(Flag::F3, acc_new & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, acc_new & 0x20 != 0); // 5 bit
                    cpu.regs.set_acc(acc_new);
                }
                // CPL Invert (Complement)
                // [0b00101111] : 0x2F
                U3::N5 => {
                    let data = !cpu.regs.get_acc();
                    cpu.regs.set_flag(Flag::HalfCarry, true);
                    cpu.regs.set_flag(Flag::Sub, true);
                    cpu.regs.set_flag(Flag::F3, data & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, data & 0x20 != 0); // 5 bit
                    cpu.regs.set_acc(data);
                }
                // SCF  Set carry flag
                // [0b00110111] : 0x37
                U3::N6 => {
                    let data = cpu.regs.get_acc();
                    cpu.regs.set_flag(Flag::F3, data & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, data & 0x20 != 0); // 5 bit
                    cpu.regs.set_flag(Flag::HalfCarry, false);
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::Carry, true);
                }
                // CCF Invert carry flag
                // [0b00111111] : 0x3F
                U3::N7 => {
                    let data = cpu.regs.get_acc();
                    cpu.regs.set_flag(Flag::F3, data & 0x08 != 0); // 3 bit
                    cpu.regs.set_flag(Flag::F5, data & 0x20 != 0); // 5 bit
                    let carry = cpu.regs.get_flag(Flag::Carry);
                    cpu.regs.set_flag(Flag::HalfCarry, carry);
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::Carry, !carry);
                }
            }
        }
        // HALT
        // [0b01110110] : 0x76
        U2::N1 if (opcode.z == U3::N6) && (opcode.y == U3::N6) => {
            cpu.halted = true;
        }
        // ---------------------------------
        // [0b01yyyzzz] instruction section
        // ---------------------------------
        // LD r[y], r[z]
        // [0b01yyyzzz]: 0x40...0x7F
        U2::N1 => {
            // LD r[y], r[z] without indirection
            if (opcode.z != U3::N6) && (opcode.y != U3::N6) {
                let from = RegName8::from_u3(opcode.z).unwrap().with_prefix(prefix);
                let to = RegName8::from_u3(opcode.y).unwrap().with_prefix(prefix);
                let tmp = cpu.regs.get_reg_8(from);
                cpu.regs.set_reg_8(to, tmp);
            } else {
                // LD (HL/IX+d/IY+d), r ; LD r, (HL/IX+d/IY+d)
                // 0x01110zzz; 0x01yyy110
                let from = if let Some(reg) = RegName8::from_u3(opcode.z) {
                    // H/L is not affected by prefix if already indirection
                    LoadOperand8::Reg(reg)
                } else {
                    if prefix == Prefix::None {
                        LoadOperand8::Indirect(cpu.regs.get_reg_16(RegName16::HL))
                    } else {
                        let d = cpu.rom_next_byte(bus) as i8;
                        LoadOperand8::Indirect(word_displacement(cpu.regs.get_reg_16(
                            RegName16::HL.with_prefix(prefix)), d))
                    }
                };
                let to = if let Some(reg) = RegName8::from_u3(opcode.y) {
                    // H/L is not affected by prefix if already indirection
                    LoadOperand8::Reg(reg)
                } else {
                    if prefix == Prefix::None {
                        LoadOperand8::Indirect(cpu.regs.get_reg_16(RegName16::HL))
                    } else {
                        let d = cpu.rom_next_byte(bus) as i8;
                        LoadOperand8::Indirect(word_displacement(cpu.regs.get_reg_16(
                            RegName16::HL.with_prefix(prefix)), d))
                    }
                };
                let data = match from {
                    LoadOperand8::Indirect(addr) => bus.read(addr),
                    LoadOperand8::Reg(reg) => cpu.regs.get_reg_8(reg),
                };
                match to {
                    LoadOperand8::Indirect(addr) => {
                        bus.write(addr, data);
                    }
                    LoadOperand8::Reg(reg) => {
                        cpu.regs.set_reg_8(reg, data);
                    }
                };
            }
        }
        // ---------------------------------
        // [0b10yyyzzz] instruction section
        // ---------------------------------
        // alu[y], operand[z-based]; 0x80...0xBF
        U2::N2 => {
            let operand = if let Some(reg) = RegName8::from_u3(opcode.z) {
                // alu[y] reg
                cpu.regs.get_reg_8(reg.with_prefix(prefix))
            } else {
                // alu[y] (HL/IX+d/IY+d)
                if prefix == Prefix::None {
                    bus.read(cpu.regs.get_reg_16(RegName16::HL))
                } else {
                    let d = cpu.rom_next_byte(bus) as i8;
                    let addr = cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix));
                    bus.read(word_displacement(addr, d))
                }
            };
            execute_alu_8(cpu, opcode.y, operand);
        }
        // ---------------------------------
        // [0b11yyyzzz] instruction section
        // ---------------------------------
        // RET cc[y]
        // [0b11yyy000] : C0; C8; D0; D8; E0; E8; F0; F8;
        U2::N3 if opcode.z == U3::N0 => {
            if cpu.regs.eval_condition(Condition::from_u3(opcode.y)) {
                // write value from stack to pc
                execute_pop_16(cpu, bus, RegName16::PC);
                clocks += 11;
            } else {
                clocks += 5;
            };
        }
        // [0b11ppq001] instruction group
        U2::N3 if opcode.z == U3::N1 => {
            match opcode.q {
                // POP (AF/BC/DE/HL/IX/IY) ; pop 16 bit register featuring A
                // [0b11pp0001]: C1; D1; E1; F1;
                U1::N0 => {
                    execute_pop_16(cpu, bus, RegName16::from_u2_af(opcode.p).with_prefix(prefix));
                }
                // [0b11pp1001] instruction group (assorted)
                U1::N1 => {
                    match opcode.p {
                        // RET ; return
                        // [0b11001001] : C9;
                        U2::N0 => {
                            execute_pop_16(cpu, bus, RegName16::PC);
                        }
                        // EXX
                        // [0b11011001] : D9;
                        U2::N1 => {
                            cpu.regs.exx();
                        }
                        // JP HL/IX/IY
                        // [0b11101001] : E9
                        U2::N2 => {
                            let addr = cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix));
                            cpu.regs.set_pc(addr);
                        }
                        // LD SP, HL/IX/IY
                        // [0b11111001] : F9
                        U2::N3 => {
                            let data = cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix));
                            cpu.regs.set_sp(data);
                        }
                    }
                }
            };
        }
        // JP cc[y], nn [timings is set to 10 anyway as showed in Z80 instruction!]
        // [0b11yyy010]: C2,CA,D2,DA,E2,EA,F2,FA
        // NOTE: Maybe timings incorrect
        U2::N3 if opcode.z == U3::N2 => {
            let addr = cpu.rom_next_word(bus);
            if cpu.regs.eval_condition(Condition::from_u3(opcode.y)) {
                cpu.regs.set_pc(addr);
            };
        }
        // [0b11yyy011] instruction group (assorted)
        U2::N3 if opcode.z == U3::N3 => {
            match opcode.y {
                // JP nn
                // [0b11000011]: C3
                U3::N0 => {
                    let addr = cpu.rom_next_word(bus);
                    cpu.regs.set_pc(addr);
                }
                // CB prefix
                U3::N1 => {
                    panic!("CB prefix passed as non-prefixed instruction");
                }
                // OUT (n), A
                // [0b11010011] : D3
                U3::N2 => {
                    let data = cpu.rom_next_byte(bus);
                    let acc = cpu.regs.get_acc();
                    // write Acc to port A*256 + operand
                    bus.write_io(((acc as u16) << 8) | data as u16, acc);
                }
                // IN A, (n)
                // [0b11011011] : DB
                U3::N3 => {
                    let data = cpu.rom_next_byte(bus);
                    let acc = cpu.regs.get_acc();
                    // read from port A*256 + operand to Acc
                    cpu.regs.set_acc(bus.read_io(((acc as u16) << 8) | data as u16));
                }
                // EX (SP), HL/IX/IY
                // [0b11100011] : E3
                U3::N4 => {
                    let reg = RegName16::HL.with_prefix(prefix);
                    let addr = cpu.regs.get_sp();
                    let tmp = bus.read_word(addr);
                    bus.write_word(addr, cpu.regs.get_reg_16(reg));
                    cpu.regs.set_reg_16(reg, tmp);
                }
                // EX DE, HL
                // [0b11101011]
                U3::N5 => {
                    let de = cpu.regs.get_reg_16(RegName16::DE);
                    let hl = cpu.regs.get_reg_16(RegName16::HL);
                    cpu.regs.set_reg_16(RegName16::DE, hl);
                    cpu.regs.set_reg_16(RegName16::HL, de);
                }
                // DI
                // [0b11110011] : F3
                U3::N6 => {
                    // skip interrupt check and reset flip-flops
                    cpu.skip_interrupt = true;
                    cpu.regs.set_iff1(false);
                    cpu.regs.set_iff2(false);
                }
                // EI
                // [0b11111011] : FB
                U3::N7 => {
                    // skip interrupt check and set flip-flops
                    cpu.skip_interrupt = true;
                    cpu.regs.set_iff1(true);
                    cpu.regs.set_iff2(true);
                }
            }
        }
        // CALL cc[y], nn
        // [0b11ccc100] : C4; CC; D4; DC; E4; EC; F4; FC
        U2::N3 if opcode.z == U3::N4 => {
            let addr = cpu.rom_next_word(bus);
            if cpu.regs.eval_condition(Condition::from_u3(opcode.y)) {
                execute_push_16(cpu, bus, RegName16::PC);
                cpu.regs.set_reg_16(RegName16::PC, addr);
                clocks += 5;
            } else {
                clocks += 3;
            };
        }
        //  [0b11ppq101] opcodes group : PUSH rp2[p], CALL nn
        U2::N3 if opcode.z == U3::N5 => {
            match opcode.q {
                // PUSH rp2[p]
                // [0b11pp0101] : C5; D5; E5; F5;
                U1::N0 => {
                    execute_push_16(cpu, bus, RegName16::from_u2_af(opcode.p).with_prefix(prefix));
                }
                U1::N1 => {
                    match opcode.p {
                        // CALL nn
                        // [0b11001101] : CD
                        U2::N0 => {
                            let addr = cpu.rom_next_word(bus);
                            execute_push_16(cpu, bus, RegName16::PC);
                            cpu.regs.set_reg_16(RegName16::PC, addr);
                        }
                        // [0b11011101] : DD
                        U2::N1 => {
                            panic!("DD prefix passed as non-prefixed instruction");
                        }
                        // [0b11101101] : ED
                        U2::N2 => {
                            panic!("ED prefix passed as non-prefixed instruction");
                        }
                        // [0b11111101] : FD
                        U2::N3 => {
                            panic!("FD prefix passed as non-prefixed instruction");
                        }
                    }
                }
            }
        }
        // alu[y] NN
        // [0b11yyy110] : C6; CE; D6; DE; E6; EE; F6; FE
        U2::N3 if opcode.z == U3::N6 => {
            let operand = cpu.rom_next_byte(bus);
            execute_alu_8(cpu, opcode.y, operand);
        }
        // RST y*8
        // [0b11yyy111]
        U2::N3 => {
            execute_push_16(cpu, bus, RegName16::PC);
            // CALL y*8
            cpu.regs.set_reg_16(RegName16::PC, (opcode.y.as_byte() as u16) << 3);
        }
    };
    if prefix == Prefix::None {
        clocks += tables::CLOCKS_NORMAL[opcode.byte as usize];
        cpu.regs.inc_r(1); // single inc
    } else {
        clocks += tables::CLOCKS_DD_FD[opcode.byte as usize];
        cpu.regs.inc_r(2); // DD or FD prefix double inc R reg
    };
    Clocks::Some(clocks)
}
