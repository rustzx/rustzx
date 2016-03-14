use super::*;
use z80::*;
use utils::*;

/// Extended instruction group (ED-prefixed)
/// Operations are assorted.
pub fn execute_extended(cpu: &mut Z80, bus: &mut Z80Bus, opcode: Opcode) -> Clocks {
    let mut clocks = 0;
    // LD A, R; LD R, A accessing R after incю So increment it twice now!
    cpu.regs.inc_r(2);
    match opcode.x {
        U2::N0 | U2::N3 => {
            // Nothing. Just nothung. Invalid opcodes.
            // But timings in table still exsist, all ok.
            // Maybe I'll  add some debug codes or something in future.
        }
        // ---------------------------------
        // [0b01yyyzzz] instruction section
        // ---------------------------------
        // Assorted operations
        U2::N1 => {
            match opcode.z {
                // IN
                // [0b01yyy000] : 40 48 50 58 60 68 70 78
                U3::N0 => {
                    // option, if y == 6 then reg = None
                    let reg = RegName8::from_u3(opcode.y);
                    // put BC on bus (this how Z80 acts on real HW) and get io data
                    let data = bus.read_io(cpu.regs.get_bc());
                    if let Some(reg) = reg {
                        cpu.regs.set_reg_8(reg, data);
                    };
                    cpu.regs.set_flag(Flag::Sub, false);
                    cpu.regs.set_flag(Flag::HalfCarry, false);
                    cpu.regs.set_flag(Flag::F3, data & 0b1000 != 0);
                    cpu.regs.set_flag(Flag::F5, data & 0b100000 != 0);
                    cpu.regs.set_flag(Flag::Zero, data == 0);
                    cpu.regs.set_flag(Flag::Sign, data & 0x80 != 0);
                    cpu.regs.set_flag(Flag::ParityOveflow,
                                       tables::PARITY_BIT[data as usize] != 0);
                }
                // OUT
                // [0b01yyy001] : 41 49 51 59 61 69 71 79
                U3::N1 => {
                    let data = if let Some(reg) = RegName8::from_u3(opcode.y) {
                        cpu.regs.get_reg_8(reg)
                    } else {
                        0
                    };
                    bus.write_io(cpu.regs.get_bc(), data);
                }
                // SBC, ADC
                U3::N2 => {
                    let prev_carry = bool_to_u8(cpu.regs.get_flag(Flag::Carry)) as u16;
                    let operand = cpu.regs.get_reg_16(RegName16::from_u2_sp(opcode.p));
                    let hl =  cpu.regs.get_hl();
                    let (carry, sub, pv, half_carry);
                    let result;
                    match opcode.q {
                        // SBC HL, rp[p]
                        U1::N0 => {
                            let (r_tmp, c1) = hl.overflowing_sub(operand);
                            let (r, c2) = r_tmp.overflowing_sub(prev_carry);
                            carry = c1 | c2;
                            result = r;
                            sub = true;
                            pv = check_sub_overflow_16(hl as i16, operand as i16) |
                                 check_sub_overflow_16(r_tmp as i16, prev_carry as i16);
                            half_carry = half_borrow_16(hl, operand) |
                                         half_borrow_16(r_tmp, prev_carry);
                        }
                        // ADC HL, rp[p]
                        U1::N1 => {
                            let (r_tmp, c1) = hl.overflowing_add(operand);
                            let (r, c2) = r_tmp.overflowing_add(prev_carry);
                            carry = c1 | c2;
                            result = r;
                            sub = false;
                            pv = check_add_overflow_16(hl as i16, operand as i16) |
                                 check_add_overflow_16(r_tmp as i16, prev_carry as i16);
                            half_carry = half_carry_16(hl, operand) |
                                         half_carry_16(r_tmp, prev_carry);
                        }
                    }
                    // set f3, f5, z, s
                    cpu.regs.set_flag(Flag::Carry, carry);
                    cpu.regs.set_flag(Flag::Sub, sub);
                    cpu.regs.set_flag(Flag::ParityOveflow, pv);
                    cpu.regs.set_flag(Flag::F3, result & 0b1000 != 0);
                    cpu.regs.set_flag(Flag::F5, result & 0b100000 != 0);
                    cpu.regs.set_flag(Flag::HalfCarry, half_carry);
                    cpu.regs.set_flag(Flag::Zero, result == 0);
                    cpu.regs.set_flag(Flag::Sign, result & 0x8000 != 0);
                    cpu.regs.set_hl(result);
                }
                // LD
                U3::N3 => {
                    let addr = cpu.rom_next_word(bus);
                    let reg = RegName16::from_u2_sp(opcode.p);
                    match opcode.q {
                        // LD (nn), rp[p]
                        U1::N0 => {
                            bus.write_word(addr, cpu.regs.get_reg_16(reg));
                        }
                        // LD rp[p], (nn)
                        U1::N1 => {
                            cpu.regs.set_reg_16(reg, bus.read_word(addr));
                        }
                    }
                }
                // NEG (A = 0 - A)
                U3::N4 => {
                    let acc = cpu.regs.get_acc();
                    let result = 0u8.wrapping_sub(acc);
                    cpu.regs.set_acc(result);
                    cpu.regs.set_flag(Flag::Sign, result & 0x80 != 0);
                    cpu.regs.set_flag(Flag::Zero, result == 0);
                    cpu.regs.set_flag(Flag::HalfCarry, half_borrow_8(0, acc));
                    cpu.regs.set_flag(Flag::ParityOveflow, acc == 0x80);
                    cpu.regs.set_flag(Flag::Sub, true);
                    cpu.regs.set_flag(Flag::Carry, acc != 0x00);
                    cpu.regs.set_flag(Flag::F3, result & 0b1000 != 0);
                    cpu.regs.set_flag(Flag::F5, result & 0b100000 != 0);
                }
                // RETN, RETI
                U3::N5 => {
                    // RETN and even RETI copy iff2 into iff1
                    let iff2 = cpu.regs.get_iff2();
                    cpu.regs.set_iff1(iff2);
                    // restore PC
                    execute_pop_16(cpu, bus, RegName16::PC);
                    if opcode.y == U3::N1 {
                        bus.reti();
                    }
                }
                // IM im[y]
                U3::N6 => {
                    cpu.int_mode =  match opcode.y {
                        U3::N0 | U3::N1 | U3::N4 | U3::N5 => {
                            IntMode::IM0
                        }
                        U3::N2 | U3::N6 => {
                            IntMode::IM1
                        }
                        U3::N3 | U3::N7 => {
                            IntMode::IM2
                        }
                    };
                }
                // Assorted - LD,Rotates, Nop
                U3::N7 => {
                    match opcode.y {
                        // LD I, A
                        U3::N0 => {
                            let acc = cpu.regs.get_acc();
                            cpu.regs.set_i(acc);
                        }
                        // LD R, A
                        U3::N1 => {
                            let acc = cpu.regs.get_acc();
                            cpu.regs.set_r(acc);
                        }
                        // LD A, I
                        U3::N2 => {
                            let i = cpu.regs.get_i();
                            cpu.regs.set_acc(i);
                        }
                        // LD A, R
                        U3::N3 => {
                            let r = cpu.regs.get_r();
                            cpu.regs.set_acc(r);
                        }
                        // RRD
                        U3::N4 => {
                            let mut acc = cpu.regs.get_acc();
                            let mut mem = bus.read(cpu.regs.get_hl());
                            // low nimble
                            let mem_lo = mem & 0x0F;
                            // mem_hi to mem_lo and clear hi nimble
                            mem = (mem >> 4) & 0x0F;
                            // acc_lo to mem_hi
                            mem = mem | ((acc << 4) & 0xF0);
                            acc = (acc & 0xF0) | mem_lo;
                            cpu.regs.set_acc(acc);
                            bus.write(cpu.regs.get_hl(), mem);
                            cpu.regs.set_flag(Flag::Sign, acc & 0x80 != 0);
                            cpu.regs.set_flag(Flag::Zero, acc == 0);
                            cpu.regs.set_flag(Flag::HalfCarry, false);
                            cpu.regs.set_flag(Flag::ParityOveflow,
                                               tables::PARITY_BIT[acc as usize] != 0);
                            cpu.regs.set_flag(Flag::Sub, false);
                            cpu.regs.set_flag(Flag::F3, acc & 0b1000 != 0);
                            cpu.regs.set_flag(Flag::F5, acc & 0b100000 != 0);

                        }
                        // RLD
                        U3::N5 => {
                            let mut acc = cpu.regs.get_acc();
                            let mut mem = bus.read(cpu.regs.get_hl());
                            // low nimble
                            let acc_lo = acc & 0x0F;
                            // mem_hi to acc_lo
                            acc = (acc & 0xF0) | ((mem >> 4) & 0x0F);
                            // mem_lo to mem_hi and tmp to mem_lo
                            mem = ((mem << 4) & 0xF0) | acc_lo;
                            cpu.regs.set_acc(acc);
                            bus.write(cpu.regs.get_hl(), mem);
                            cpu.regs.set_flag(Flag::Sign, acc & 0x80 != 0);
                            cpu.regs.set_flag(Flag::Zero, acc == 0);
                            cpu.regs.set_flag(Flag::HalfCarry, false);
                            cpu.regs.set_flag(Flag::ParityOveflow,
                                               tables::PARITY_BIT[acc as usize] != 0);
                            cpu.regs.set_flag(Flag::Sub, false);
                            cpu.regs.set_flag(Flag::F3, acc & 0b1000 != 0);
                            cpu.regs.set_flag(Flag::F5, acc & 0b100000 != 0);
                        }
                        // NOP
                        U3::N6 | U3::N7 => {
                            // No operation
                        }
                    }
                }
            }
        }
        // ---------------------------------
        // [0b10yyyzzz] instruction section
        // ---------------------------------
        // Block instructions
        U2::N2 => {
            match opcode.z {
                // LD Block group
                U3::N0 => {
                    match opcode.y {
                        // LDI
                        U3::N4 => execute_ldi_ldd(cpu, bus, BlockDir::Inc),
                        // LDD
                        U3::N5 => execute_ldi_ldd(cpu, bus, BlockDir::Dec),
                        // LDIR
                        U3::N6 => {
                            execute_ldi_ldd(cpu, bus, BlockDir::Inc);
                            if cpu.regs.get_reg_16(RegName16::BC) != 0 {
                                cpu.regs.dec_pc(2);
                                clocks += 21;
                            } else {
                                clocks += 16;
                            };
                        }
                        // LDDR
                        U3::N7 => {
                            execute_ldi_ldd(cpu, bus, BlockDir::Dec);
                            if cpu.regs.get_reg_16(RegName16::BC) != 0 {
                                cpu.regs.dec_pc(2);
                                clocks += 21;
                            } else {
                                clocks += 16;
                            };
                        }
                        // No operation
                        _ => {},
                    }
                }
                // CP Block group
                U3::N1 => {
                    match opcode.y {
                        // CPI
                        U3::N4 => {
                            execute_cpi_cpd(cpu, bus, BlockDir::Inc);
                        }
                        // CPD
                        U3::N5 => {
                            execute_cpi_cpd(cpu, bus, BlockDir::Dec);
                        }
                        // CPIR
                        U3::N6 => {
                            let result = execute_cpi_cpd(cpu, bus, BlockDir::Inc);
                            if (cpu.regs.get_reg_16(RegName16::BC) != 0) & (!result) {
                                cpu.regs.dec_pc(2);
                                clocks += 21;
                            } else {
                                clocks += 16;
                            };
                        }
                        // CPDR
                        U3::N7 => {
                            let result = execute_cpi_cpd(cpu, bus, BlockDir::Dec);
                            if (cpu.regs.get_reg_16(RegName16::BC) != 0) & (!result) {
                                cpu.regs.dec_pc(2);
                                clocks += 21;
                            } else {
                                clocks += 16;
                            };
                        }
                        // No operation
                        _ => {},
                    }
                }
                // IN Block group
                U3::N2 => {
                    match opcode.y {
                        // INI
                        U3::N4 => execute_ini_ind(cpu, bus, BlockDir::Inc),
                        // IND
                        U3::N5 => execute_ini_ind(cpu, bus, BlockDir::Dec),
                        // INIR
                        U3::N6 => {
                            execute_ini_ind(cpu, bus, BlockDir::Inc);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                cpu.regs.dec_pc(2);
                                clocks += 21
                            } else {
                                clocks += 16;
                            };
                        }
                        // INDR
                        U3::N7 => {
                            execute_ini_ind(cpu, bus, BlockDir::Dec);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                cpu.regs.dec_pc(2);
                                clocks += 21;
                            } else {
                                clocks += 16;
                            };
                        }
                        // No operation
                        _ => {},
                    }
                }
                // Out Block group
                U3::N3 => {
                    match opcode.y {
                        // OUTI
                        U3::N4 => execute_outi_outd(cpu, bus, BlockDir::Inc),
                        // OUTD
                        U3::N5 => execute_outi_outd(cpu, bus, BlockDir::Dec),
                        // OTIR
                        U3::N6 => {
                            execute_outi_outd(cpu, bus, BlockDir::Inc);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                cpu.regs.dec_pc(2);
                                clocks += 21
                            } else {
                                clocks += 16;
                            };
                        }
                        // OTDR
                        U3::N7 => {
                            execute_outi_outd(cpu, bus, BlockDir::Dec);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                cpu.regs.dec_pc(2);
                                clocks += 21;
                            } else {
                                clocks += 16;
                            };
                        }
                        // No operation
                        _ => {},
                    }
                }
                // No operation
                _ => {},
            }
        }
    }
    clocks += tables::CLOCKS_ED[opcode.byte as usize];
    Clocks::Some(clocks)
}
