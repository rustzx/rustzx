use crate::{
    utils::{bool_to_u8, make_word, split_word, word_displacement, Clocks, U1, U2, U3},
    z80::{
        opcodes::{execute_alu_8, execute_pop_16, execute_push_16, LoadOperand8, Opcode},
        tables::{
            lookup16_r12, lookup8_r12, F3F5_TABLE, HALF_CARRY_ADD_TABLE, HALF_CARRY_SUB_TABLE,
            SZF3F5_TABLE, SZPF3F5_TABLE,
        },
        Condition, Flag, Prefix, RegName16, RegName8, Z80Bus, FLAG_CARRY, FLAG_F3, FLAG_F5,
        FLAG_HALF_CARRY, FLAG_PV, FLAG_SIGN, FLAG_SUB, FLAG_ZERO, Z80,
    },
};

/// normal execution group, can be modified with prefixes DD, FD, providing
/// DD OPCODE [NN], FD OPCODE [NN] instruction group
///
/// Opcode matching organised based on
/// [document](http://www.z80.info/decoding.htm) by Cristian Dinu
///
/// DAA algorithm
/// [link](http://www.worldofspectrum.org/faq/reference/z80reference.htm#DAA)
pub fn execute_normal(cpu: &mut Z80, bus: &mut dyn Z80Bus, opcode: Opcode, prefix: Prefix) {
    // 2 first bits of opcode
    match opcode.x {
        // ---------------------------------
        // [0b00yyyzzz] instruction section
        // ---------------------------------
        // [0b00yyy000] instruction group (NOP, EX, DJNZ, JR)
        U2::N0 if opcode.z == U3::N0 => {
            match opcode.y {
                // NOP, 4 clocks
                // [0b00000000] = 0x00
                U3::N0 => {}
                // EX AF, AF', 4 clocks
                // [0b00001000] = 0x08
                U3::N1 => {
                    cpu.regs.swap_af_alt();
                }
                // DJNZ offset;   (4 + 1 + 3) + [5] = 8 or 13 clocks
                // [0b00010000] = 0x10
                U3::N2 => {
                    bus.wait_no_mreq(cpu.regs.get_ir(), Clocks(1));
                    // emulate read byte without pc shift
                    let offset = bus.read(cpu.regs.get_pc(), Clocks(3)) as i8;
                    // preform jump if needed
                    if cpu.regs.dec_reg_8(RegName8::B, 1) != 0 {
                        bus.wait_loop(cpu.regs.get_pc(), Clocks(5));
                        cpu.regs.shift_pc(offset);
                    };
                    // inc pc, what left after reading displacement
                    cpu.regs.inc_pc(1);
                }
                // JR offset
                // [0b00011000] = 0x18
                U3::N3 => {
                    // same rules as DJNZ
                    let offset = bus.read(cpu.regs.get_pc(), Clocks(3)) as i8;
                    bus.wait_loop(cpu.regs.get_pc(), Clocks(5));
                    cpu.regs.shift_pc(offset);
                    cpu.regs.inc_pc(1);
                }
                // JR condition[y-4] displacement; 4 + 3 + [5] = 7/12 clocks
                // NZ [0b00100000], Z [0b00101000] NC [0b00110000] C [0b00111000]
                U3::N4 | U3::N5 | U3::N6 | U3::N7 => {
                    // 0x20, 0x28, 0x30, 0x38
                    let offset = bus.read(cpu.regs.get_pc(), Clocks(3)) as i8;
                    // y in range 4..7
                    let cnd = Condition::from_u3(U3::from_byte(opcode.y.as_byte() - 4, 0));
                    if cpu.regs.eval_condition(cnd) {
                        bus.wait_loop(cpu.regs.get_pc(), Clocks(5));
                        cpu.regs.shift_pc(offset);
                    };
                    // inc pc, which left after reading displacement
                    cpu.regs.inc_pc(1);
                }
            };
        }
        // [0b00ppq001] instruction group (LD, ADD)
        U2::N0 if opcode.z == U3::N1 => {
            match opcode.q {
                // LD rp[p], nn, 4 +  3 + 3 = 10 clcocks
                // [0b00pp0001] : 0x01, 0x11, 0x21, 0x31
                U1::N0 => {
                    let reg = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
                    let data = cpu.fetch_word(bus, Clocks(3));
                    cpu.regs.set_reg_16(reg, data);
                }
                // ADD HL/IX/IY, ss ; ss - 16 bit with sp set
                // [0b00pp1001] : 0x09; 0x19; 0x29; 0x39
                U1::N1 => {
                    bus.wait_loop(cpu.regs.get_ir(), Clocks(7));
                    let reg_operand = RegName16::from_u2_sp(opcode.p).with_prefix(prefix);
                    let reg_acc = RegName16::HL.with_prefix(prefix);
                    let acc = cpu.regs.get_reg_16(reg_acc);
                    let operand = cpu.regs.get_reg_16(reg_operand);
                    let temp: u32 = (acc as u32).wrapping_add(operand as u32);
                    // watch tables module
                    let lookup = lookup16_r12(acc, operand, temp as u16);
                    // get last flags, reset affected by instruction
                    let mut flags = cpu.regs.get_flags() & (FLAG_ZERO | FLAG_PV | FLAG_SIGN);
                    flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
                    flags |= bool_to_u8(temp > 0xFFFF) * FLAG_CARRY;
                    flags |= F3F5_TABLE[((temp >> 8) as u8) as usize];
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_reg_16(reg_acc, temp as u16);
                }
            };
        }
        // [0b00ppq010] instruction group (LD INDIRECT)
        U2::N0 if opcode.z == U3::N2 => {
            match opcode.q {
                // LD (BC), A  // 4 + 3 = 7 clocks
                // [0b00000010] : 0x02
                U1::N0 if opcode.p == U2::N0 => {
                    bus.write(cpu.regs.get_bc(), cpu.regs.get_acc(), Clocks(3));
                }
                // LD (DE), A // 4 + 3 = 7 clocks
                // [0b00010010] : 0x12
                U1::N0 if opcode.p == U2::N1 => {
                    bus.write(cpu.regs.get_de(), cpu.regs.get_acc(), Clocks(3));
                }
                // LD (nn), HL/IX/IY // 4 + 3 + 3 + 3 + 3 = 16 clocks
                // [0b00100010] : 0x22
                U1::N0 if opcode.p == U2::N2 => {
                    let addr = cpu.fetch_word(bus, Clocks(3));
                    let reg = RegName16::HL.with_prefix(prefix);
                    bus.write_word(addr, cpu.regs.get_reg_16(reg), Clocks(3));
                }
                // LD (nn), A // 4 + 3 + 3 + 3 = 13 clocks
                // [0b00110010] : 0x32
                U1::N0 => {
                    let addr = cpu.fetch_word(bus, Clocks(3));
                    bus.write(addr, cpu.regs.get_acc(), Clocks(3));
                }
                // LD A, (BC) // 4 + 3 = 7 clocks
                // [0b00001010] : 0x0A
                U1::N1 if opcode.p == U2::N0 => {
                    let addr = cpu.regs.get_bc();
                    cpu.regs.set_acc(bus.read(addr, Clocks(3)));
                }
                // LD A, (DE) // 4 + 3 = 7 clocks
                // [0b00011010] : 0x1A
                U1::N1 if opcode.p == U2::N1 => {
                    let addr = cpu.regs.get_de();
                    cpu.regs.set_acc(bus.read(addr, Clocks(3)));
                }
                // LD HL/IX/IY, (nn) // 4 + 3 + 3 + 3 + 3 = 16 clocks
                // [0b00101010] : 0x2A
                U1::N1 if opcode.p == U2::N2 => {
                    let addr = cpu.fetch_word(bus, Clocks(3));
                    let reg = RegName16::HL.with_prefix(prefix);
                    cpu.regs.set_reg_16(reg, bus.read_word(addr, Clocks(3)));
                }
                // LD A, (nn) // 4 + 3 + 3 + 3 = 13 clocks
                // [0b00111010] : 0x3A
                U1::N1 => {
                    let addr = cpu.fetch_word(bus, Clocks(3));
                    cpu.regs.set_acc(bus.read(addr, Clocks(3)));
                }
            };
        }
        // [0b00ppq011] instruction group (INC, DEC)
        U2::N0 if opcode.z == U3::N3 => {
            bus.wait_loop(cpu.regs.get_ir(), Clocks(2));
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
                // INC r[y], DEC r[y] ; IX and IY also used
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
                    cpu.regs.get_hl()
                } else {
                    // we have INC/DEC (IX/IY + d)
                    let d = bus.read(cpu.regs.get_pc(), Clocks(3)) as i8;
                    bus.wait_loop(cpu.regs.get_pc(), Clocks(5));
                    cpu.regs.inc_pc(1);
                    word_displacement(cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)), d)
                };
                // read data
                data = bus.read(addr, Clocks(3));
                bus.wait_no_mreq(addr, Clocks(1));
                operand = LoadOperand8::Indirect(addr);
            };
            // ------------
            //   execute
            // ------------
            // carry unaffected
            let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
            if opcode.z == U3::N4 {
                // INC
                result = data.wrapping_add(1);
                flags |= bool_to_u8(data == 0x7F) * FLAG_PV;
                let lookup = lookup8_r12(data, 1, result);
                flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
            } else {
                // DEC
                result = data.wrapping_sub(1);
                flags |= FLAG_SUB;
                flags |= bool_to_u8(data == 0x80) * FLAG_PV;
                let lookup = lookup8_r12(data, 1, result);
                flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
            }
            flags |= SZF3F5_TABLE[result as usize];
            cpu.regs.set_flags(flags);
            // ------------
            //  write data
            // ------------
            match operand {
                LoadOperand8::Indirect(addr) => {
                    bus.write(addr, result, Clocks(3));
                }
                LoadOperand8::Reg(reg) => {
                    cpu.regs.set_reg_8(reg, result);
                }
            };
            // Clocks:
            // Direct : 4
            // HL : 4 + 3 + 1 + 3 = 11
            // XY+d : 4 + 4 + 3 + 5 + 3 + 1 + 3 = 23
        }
        // [0b00yyy110] instruction group (LD R, N 8 bit) :
        // 0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E
        U2::N0 if opcode.z == U3::N6 => {
            let operand = if let Some(reg) = RegName8::from_u3(opcode.y) {
                // Direct LD R, N
                LoadOperand8::Reg(reg.with_prefix(prefix))
            } else {
                // INDIRECT LD (HL/IX+d/IY+d), N <PREFIX>[0b00110110] : 0x36
                let addr = if prefix == Prefix::None {
                    // LD (HL)
                    cpu.regs.get_hl()
                } else {
                    // LD (IX+d/ IY+d)
                    let d = cpu.fetch_byte(bus, Clocks(3)) as i8;
                    word_displacement(cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)), d)
                };
                LoadOperand8::Indirect(addr)
            };
            // Read const operand
            let data = bus.read(cpu.regs.get_pc(), Clocks(3));
            // if non-prefixed and there is no indirection
            if prefix != Prefix::None {
                if let LoadOperand8::Indirect(_) = operand {
                    bus.wait_loop(cpu.regs.get_pc(), Clocks(2));
                }
            }
            cpu.regs.inc_pc(1);
            // write to bus or reg
            match operand {
                LoadOperand8::Indirect(addr) => {
                    bus.write(addr, data, Clocks(3));
                }
                LoadOperand8::Reg(reg) => {
                    cpu.regs.set_reg_8(reg, data);
                }
            };
            // Clocks:
            // Direct: 4 + 3 = 7
            // HL: 4 + 3 + 3 = 10
            // XY+d: 4 + 4 + 3 + 3 + 2 + 3 = 19
        }
        // [0b00yyy111] instruction group (Assorted)
        U2::N0 => {
            match opcode.y {
                // RLCA ; Rotate left; msb will become lsb; carry = msb
                // [0b00000111] : 0x07
                U3::N0 => {
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x80) != 0;
                    data <<= 1;
                    if carry {
                        data |= 1;
                    } else {
                        data &= 0xFE;
                    };
                    let mut flags = cpu.regs.get_flags() & (FLAG_PV | FLAG_SIGN | FLAG_ZERO);
                    flags |= bool_to_u8(carry) * FLAG_CARRY;
                    flags |= F3F5_TABLE[data as usize];
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_acc(data);
                }
                // RRCA ; Rotate right; lsb will become msb; carry = lsb
                // [0b00001111] : 0x0F
                U3::N1 => {
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x01) != 0;
                    data >>= 1;
                    if carry {
                        data |= 0x80;
                    } else {
                        data &= 0x7F;
                    };
                    let mut flags = cpu.regs.get_flags() & (FLAG_PV | FLAG_SIGN | FLAG_ZERO);
                    flags |= bool_to_u8(carry) * FLAG_CARRY;
                    flags |= F3F5_TABLE[data as usize];
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_acc(data);
                }
                // RLA Rotate left trough carry
                // [0b00010111]: 0x17
                U3::N2 => {
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x80) != 0;
                    data <<= 1;
                    if cpu.regs.get_flag(Flag::Carry) {
                        data |= 1;
                    } else {
                        data &= 0xFE;
                    };
                    let mut flags = cpu.regs.get_flags() & (FLAG_PV | FLAG_SIGN | FLAG_ZERO);
                    flags |= bool_to_u8(carry) * FLAG_CARRY;
                    flags |= F3F5_TABLE[data as usize];
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_acc(data);
                }
                // RRA Rotate right trough carry
                // [0b00011111] : 0x1F
                U3::N3 => {
                    let mut data = cpu.regs.get_acc();
                    let carry = (data & 0x01) != 0;
                    data >>= 1;
                    if cpu.regs.get_flag(Flag::Carry) {
                        data |= 0x80;
                    } else {
                        data &= 0x7F;
                    };
                    let mut flags = cpu.regs.get_flags() & (FLAG_PV | FLAG_SIGN | FLAG_ZERO);
                    flags |= bool_to_u8(carry) * FLAG_CARRY;
                    flags |= F3F5_TABLE[data as usize];
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_acc(data);
                }
                // DAA [0b00100111] [link to the algorithm in header]
                U3::N4 => {
                    let acc = cpu.regs.get_acc();
                    let old_flags = cpu.regs.get_flags();
                    let mut flags = old_flags & FLAG_SUB;
                    let mut correction;
                    if (acc > 0x99) || ((old_flags & FLAG_CARRY) != 0) {
                        correction = 0x60_u8;
                        flags |= FLAG_CARRY;
                    } else {
                        correction = 0x00_u8;
                    };
                    if ((acc & 0x0F) > 0x09) || ((old_flags & FLAG_HALF_CARRY) != 0) {
                        correction |= 0x06;
                    }
                    let acc_new = if (old_flags & FLAG_SUB) == 0 {
                        let lookup = lookup8_r12(acc, correction, acc.wrapping_add(correction));
                        flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
                        acc.wrapping_add(correction)
                    } else {
                        let lookup = lookup8_r12(acc, correction, acc.wrapping_sub(correction));
                        flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
                        acc.wrapping_sub(correction)
                    };
                    flags |= SZPF3F5_TABLE[acc_new as usize];
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_acc(acc_new);
                }
                // CPL Invert (Complement)
                // [0b00101111] : 0x2F
                U3::N5 => {
                    let data = !cpu.regs.get_acc();
                    let mut flags = cpu.regs.get_flags() & !(FLAG_F3 | FLAG_F5);
                    flags |= FLAG_HALF_CARRY | FLAG_SUB | F3F5_TABLE[data as usize];
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_acc(data);
                }
                // SCF  Set carry flag
                // [0b00110111] : 0x37
                U3::N6 => {
                    let data = cpu.regs.get_acc();
                    let mut flags = cpu.regs.get_flags() & (FLAG_ZERO | FLAG_PV | FLAG_SIGN);
                    flags |= F3F5_TABLE[data as usize] | FLAG_CARRY;
                    cpu.regs.set_flags(flags);
                }
                // CCF Invert carry flag
                // [0b00111111] : 0x3F
                U3::N7 => {
                    let data = cpu.regs.get_acc();
                    let old_carry = (cpu.regs.get_flags() & FLAG_CARRY) != 0;
                    let mut flags = cpu.regs.get_flags() & (FLAG_SIGN | FLAG_PV | FLAG_ZERO);
                    flags |= F3F5_TABLE[data as usize];
                    flags |= bool_to_u8(old_carry) * FLAG_HALF_CARRY;
                    flags |= bool_to_u8(!old_carry) * FLAG_CARRY;
                    cpu.regs.set_flags(flags);
                }
            }
        }
        // HALT
        // [0b01110110] : 0x76
        U2::N1 if (opcode.z == U3::N6) && (opcode.y == U3::N6) => {
            cpu.halted = true;
            bus.halt(true);
            cpu.regs.dec_pc(1);
        }
        // ---------------------------------
        // [0b01yyyzzz] instruction section
        // ---------------------------------
        // From memory to register
        // LD r[y], (HL/IX+d/IY+d)
        U2::N1 if (opcode.z == U3::N6) => {
            let src_addr = if prefix == Prefix::None {
                cpu.regs.get_hl()
            } else {
                let d = bus.read(cpu.regs.get_pc(), Clocks(3)) as i8;
                bus.wait_loop(cpu.regs.get_pc(), Clocks(5));
                cpu.regs.inc_pc(1);
                word_displacement(cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)), d)
            };
            cpu.regs.set_reg_8(
                RegName8::from_u3(opcode.y).unwrap(),
                bus.read(src_addr, Clocks(3)),
            );
            // Clocks:
            // HL: <4> + 3 = 7
            // XY+d: <[4] + 4> + [3 + 5] + 3 = 19
        }
        // LD (HL/IX+d/IY+d), r[z]
        U2::N1 if (opcode.y == U3::N6) => {
            let dst_addr = if prefix == Prefix::None {
                cpu.regs.get_hl()
            } else {
                let d = bus.read(cpu.regs.get_pc(), Clocks(3)) as i8;
                bus.wait_loop(cpu.regs.get_pc(), Clocks(5));
                cpu.regs.inc_pc(1);
                word_displacement(cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)), d)
            };
            bus.write(
                dst_addr,
                cpu.regs.get_reg_8(RegName8::from_u3(opcode.z).unwrap()),
                Clocks(3),
            );
            // Clocks:
            // HL: 4 + 3 = 7
            // XY+d: 4 + 4 + 3 + 5 + 3 = 19
        }
        // LD r[y], r[z]
        U2::N1 => {
            let from = RegName8::from_u3(opcode.z).unwrap().with_prefix(prefix);
            let to = RegName8::from_u3(opcode.y).unwrap().with_prefix(prefix);
            let tmp = cpu.regs.get_reg_8(from);
            cpu.regs.set_reg_8(to, tmp);
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
                    bus.read(cpu.regs.get_hl(), Clocks(3))
                } else {
                    let d = bus.read(cpu.regs.get_pc(), Clocks(3)) as i8;
                    bus.wait_loop(cpu.regs.get_pc(), Clocks(5));
                    cpu.regs.inc_pc(1);
                    let addr = word_displacement(
                        cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix)),
                        d,
                    );
                    bus.read(addr, Clocks(3))
                }
            };
            execute_alu_8(cpu, opcode.y, operand);
            // Clocks:
            // Direct: 4
            // HL: 4 + 3
            // XY+d: 4 + 4 + 3 + 5 + 3 = 19
        }
        // ---------------------------------
        // [0b11yyyzzz] instruction section
        // ---------------------------------
        // RET cc[y]
        // [0b11yyy000] : C0; C8; D0; D8; E0; E8; F0; F8;
        U2::N3 if opcode.z == U3::N0 => {
            bus.wait_no_mreq(cpu.regs.get_ir(), Clocks(1));
            if cpu.regs.eval_condition(Condition::from_u3(opcode.y)) {
                // write value from stack to pc
                execute_pop_16(cpu, bus, RegName16::PC, Clocks(3));
            };
            // Clocks:
            // 4 + 1 + [3 + 3] = 5/11
        }
        // [0b11ppq001] instruction group
        U2::N3 if opcode.z == U3::N1 => {
            match opcode.q {
                // POP (AF/BC/DE/HL/IX/IY) ; pop 16 bit register featuring A
                // [0b11pp0001]: C1; D1; E1; F1;
                U1::N0 => {
                    execute_pop_16(
                        cpu,
                        bus,
                        RegName16::from_u2_af(opcode.p).with_prefix(prefix),
                        Clocks(3),
                    );
                    // Clocks:
                    // [4] + 4 + 3 + 3 = 10 / 14
                }
                // [0b11pp1001] instruction group (assorted)
                U1::N1 => {
                    match opcode.p {
                        // RET ; return
                        // [0b11001001] : C9;
                        U2::N0 => {
                            execute_pop_16(cpu, bus, RegName16::PC, Clocks(3));
                            // Clocks: 10
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
                            bus.wait_loop(cpu.regs.get_ir(), Clocks(2));
                            let data = cpu.regs.get_reg_16(RegName16::HL.with_prefix(prefix));
                            cpu.regs.set_sp(data);
                        }
                    }
                }
            };
        }
        // JP cc[y], nn
        // [0b11yyy010]: C2,CA,D2,DA,E2,EA,F2,FA
        U2::N3 if opcode.z == U3::N2 => {
            let addr = cpu.fetch_word(bus, Clocks(3));
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
                    let addr = cpu.fetch_word(bus, Clocks(3));
                    cpu.regs.set_pc(addr);
                }
                // CB prefix
                U3::N1 => {
                    panic!("CB prefix passed as non-prefixed instruction");
                }
                // OUT (n), A
                // [0b11010011] : D3
                U3::N2 => {
                    let data = cpu.fetch_byte(bus, Clocks(3));
                    let acc = cpu.regs.get_acc();
                    // write Acc to port A*256 + operand
                    bus.write_io(((acc as u16) << 8) | data as u16, acc);
                }
                // IN A, (n)
                // [0b11011011] : DB
                U3::N3 => {
                    let data = cpu.fetch_byte(bus, Clocks(3));
                    let acc = cpu.regs.get_acc();
                    // read from port A*256 + operand to Acc
                    cpu.regs
                        .set_acc(bus.read_io(((acc as u16) << 8) | (data as u16)));
                }
                // EX (SP), HL/IX/IY
                // [0b11100011] : E3
                U3::N4 => {
                    let reg = RegName16::HL.with_prefix(prefix);
                    let addr = cpu.regs.get_sp();
                    let tmp = bus.read_word(addr, Clocks(3));
                    bus.wait_no_mreq(addr.wrapping_add(1), Clocks(1));
                    let (h, l) = split_word(cpu.regs.get_reg_16(reg));
                    bus.write(addr.wrapping_add(1), h, Clocks(3));
                    bus.write(addr, l, Clocks(3));
                    // bus.write_word(addr, cpu.regs.get_reg_16(reg), Clocks(3));
                    bus.wait_loop(addr, Clocks(2));
                    cpu.regs.set_reg_16(reg, tmp);
                    // Clocks: [4] + 4 + (3 + 3) + 1 + (3 + 3) + 2 = 23 or 19
                }
                // EX DE, HL
                // [0b11101011]
                U3::N5 => {
                    let de = cpu.regs.get_de();
                    let hl = cpu.regs.get_hl();
                    cpu.regs.set_de(hl);
                    cpu.regs.set_hl(de);
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
            let addr_l = cpu.fetch_byte(bus, Clocks(3));
            let addr_h = bus.read(cpu.regs.get_pc(), Clocks(3));
            let addr = make_word(addr_h, addr_l);
            if cpu.regs.eval_condition(Condition::from_u3(opcode.y)) {
                bus.wait_no_mreq(cpu.regs.get_pc(), Clocks(1));
                cpu.regs.inc_pc(1);
                execute_push_16(cpu, bus, RegName16::PC, Clocks(3));
                cpu.regs.set_pc(addr);
            } else {
                cpu.regs.inc_pc(1);
            }
        }
        // [0b11ppq101] opcodes group : PUSH rp2[p], CALL nn
        U2::N3 if opcode.z == U3::N5 => {
            match opcode.q {
                // PUSH rp2[p]
                // [0b11pp0101] : C5; D5; E5; F5;
                U1::N0 => {
                    bus.wait_no_mreq(cpu.regs.get_ir(), Clocks(1));
                    execute_push_16(
                        cpu,
                        bus,
                        RegName16::from_u2_af(opcode.p).with_prefix(prefix),
                        Clocks(3),
                    );
                }
                U1::N1 => {
                    match opcode.p {
                        // CALL nn
                        // [0b11001101] : CD
                        U2::N0 => {
                            let addr_l = cpu.fetch_byte(bus, Clocks(3));
                            let addr_h = bus.read(cpu.regs.get_pc(), Clocks(3));
                            let addr = make_word(addr_h, addr_l);
                            bus.wait_no_mreq(cpu.regs.get_pc(), Clocks(1));
                            cpu.regs.inc_pc(1);
                            execute_push_16(cpu, bus, RegName16::PC, Clocks(3));
                            cpu.regs.set_pc(addr);
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
            let operand = cpu.fetch_byte(bus, Clocks(3));
            execute_alu_8(cpu, opcode.y, operand);
        }
        // RST y*8
        // [0b11yyy111]
        U2::N3 => {
            bus.wait_no_mreq(cpu.regs.get_ir(), Clocks(1));
            execute_push_16(cpu, bus, RegName16::PC, Clocks(3));
            // CALL y*8
            cpu.regs
                .set_reg_16(RegName16::PC, (opcode.y.as_byte() as u16) * 8);
            // 4 + 1 + 3 + 3 = 11
        }
    };
}
