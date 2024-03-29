use crate::{
    opcode::{
        execute_cpi_cpd, execute_ini_ind, execute_ldi_ldd, execute_outi_outd, execute_pop_16,
        BlockDir, Opcode, Prefix,
    },
    registers::BlockIoOpcode,
    smallnum::{U1, U2, U3},
    tables::{
        lookup16_r12, lookup8_r12, HALF_CARRY_ADD_TABLE, HALF_CARRY_SUB_TABLE, OVERFLOW_ADD_TABLE,
        OVERFLOW_SUB_TABLE, SZF3F5_TABLE, SZPF3F5_TABLE,
    },
    IntMode, RegName16, RegName8, Z80Bus, FLAG_CARRY, FLAG_PV, FLAG_SUB, FLAG_ZERO, Z80,
};

/// Extended instruction group (ED-prefixed)
/// (assorted operations)
pub fn execute_extended(cpu: &mut Z80, bus: &mut impl Z80Bus, opcode: Opcode) {
    match opcode.x {
        U2::N0 | U2::N3 => {
            bus.process_unknown_opcode(Prefix::ED, opcode);
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
                    let reg = RegName8::from_u3(opcode.y);
                    cpu.regs.set_mem_ptr(cpu.regs.get_bc().wrapping_add(1));
                    let data = bus.read_io(cpu.regs.get_bc());
                    if let Some(reg) = reg {
                        cpu.regs.set_reg_8(reg, data);
                    };
                    let flags = cpu.regs.get_flags() & FLAG_CARRY | SZPF3F5_TABLE[data as usize];
                    cpu.regs.set_flags(flags);
                }
                // OUT
                // [0b01yyy001] : 41 49 51 59 61 69 71 79
                U3::N1 => {
                    cpu.regs.set_mem_ptr(cpu.regs.get_bc().wrapping_add(1));
                    let data = match RegName8::from_u3(opcode.y) {
                        Some(reg) => cpu.regs.get_reg_8(reg),
                        None => 0,
                    };
                    bus.write_io(cpu.regs.get_bc(), data);
                }
                // SBC, ADC
                // [0b0ppq010]
                U3::N2 => {
                    bus.wait_loop(cpu.regs.get_ir(), 7);
                    let with_carry = (cpu.regs.get_flags() & FLAG_CARRY) != 0;
                    let operand = cpu.regs.get_reg_16(RegName16::from_u2_sp(opcode.p));
                    let hl = cpu.regs.get_hl();
                    let mut flags = 0u8;
                    let result = match opcode.q {
                        // SBC HL, rp[p]
                        U1::N0 => {
                            cpu.regs.set_mem_ptr(cpu.regs.get_hl().wrapping_add(1));
                            let result = (hl as u32)
                                .wrapping_sub(operand as u32)
                                .wrapping_sub(with_carry as u32);
                            let lookup = lookup16_r12(hl, operand, result as u16);
                            flags |= OVERFLOW_SUB_TABLE[(lookup >> 4) as usize];
                            flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
                            flags |= FLAG_SUB;
                            result
                        }
                        // ADC HL, rp[p]
                        U1::N1 => {
                            cpu.regs.set_mem_ptr(cpu.regs.get_hl().wrapping_add(1));
                            let result = (hl as u32)
                                .wrapping_add(operand as u32)
                                .wrapping_add(with_carry as u32);
                            let lookup = lookup16_r12(hl, operand, result as u16);
                            flags |= OVERFLOW_ADD_TABLE[(lookup >> 4) as usize];
                            flags |= HALF_CARRY_ADD_TABLE[(lookup & 0x07) as usize];
                            result
                        }
                    };
                    flags |= (result > 0xFFFF) as u8 * FLAG_CARRY;
                    flags |= SZF3F5_TABLE[((result >> 8) as u8) as usize];
                    flags &= !FLAG_ZERO;
                    flags |= ((result as u16) == 0) as u8 * FLAG_ZERO;
                    cpu.regs.set_flags(flags);
                    cpu.regs.set_hl(result as u16);
                    // Clocks 4 + 4 + 7 = 15
                }
                // LD
                U3::N3 => {
                    let addr = cpu.fetch_word(bus, 3);
                    let reg = RegName16::from_u2_sp(opcode.p);
                    match opcode.q {
                        // LD (nn), rp[p]
                        U1::N0 => {
                            bus.write_word(addr, cpu.regs.get_reg_16(reg), 3);
                        }
                        // LD rp[p], (nn)
                        U1::N1 => {
                            let val = bus.read_word(addr, 3);
                            cpu.regs.set_reg_16(reg, val);
                        }
                    }
                    cpu.regs.set_mem_ptr(addr.wrapping_add(1));
                }
                // NEG (A = 0 - A)
                U3::N4 => {
                    let acc = cpu.regs.get_acc();
                    let result = 0u8.wrapping_sub(acc);
                    cpu.regs.set_acc(result);
                    let mut flags = FLAG_SUB;
                    flags |= SZF3F5_TABLE[result as usize];
                    let lookup = lookup8_r12(0, acc, result);
                    flags |= HALF_CARRY_SUB_TABLE[(lookup & 0x07) as usize];
                    flags |= (acc == 0x80) as u8 * FLAG_PV;
                    flags |= (acc != 0x00) as u8 * FLAG_CARRY;
                    cpu.regs.set_flags(flags);
                }
                // RETN, RETI
                U3::N5 => {
                    // RETN and even RETI should copy iff2 into iff1
                    let iff2 = cpu.regs.get_iff2();
                    cpu.regs.set_iff1(iff2);
                    execute_pop_16(cpu, bus, RegName16::PC, 3);
                    cpu.regs.set_mem_ptr(cpu.regs.get_pc());
                    if opcode.y == U3::N1 {
                        bus.reti();
                    }
                }
                // IM im[y]
                U3::N6 => {
                    cpu.int_mode = match opcode.y {
                        U3::N0 | U3::N1 | U3::N4 | U3::N5 => IntMode::Im0,
                        U3::N2 | U3::N6 => IntMode::Im1,
                        U3::N3 | U3::N7 => IntMode::Im2,
                    };
                }
                // Assorted - LD, rotations, nop's
                U3::N7 => {
                    match opcode.y {
                        // LD I, A
                        U3::N0 => {
                            bus.wait_no_mreq(cpu.regs.get_ir(), 1);
                            let acc = cpu.regs.get_acc();
                            cpu.regs.set_i(acc);
                        }
                        // LD R, A
                        U3::N1 => {
                            bus.wait_no_mreq(cpu.regs.get_ir(), 1);
                            let acc = cpu.regs.get_acc();
                            cpu.regs.set_r(acc);
                        }
                        // LD A, I
                        U3::N2 => {
                            bus.wait_no_mreq(cpu.regs.get_ir(), 1);
                            let iff2 = cpu.regs.get_iff2();
                            let i = cpu.regs.get_i();
                            cpu.regs.set_acc(i);
                            let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
                            flags |= SZF3F5_TABLE[i as usize];
                            flags |= iff2 as u8 * FLAG_PV;
                            cpu.regs.set_flags(flags);
                        }
                        // LD A, R
                        U3::N3 => {
                            bus.wait_no_mreq(cpu.regs.get_ir(), 1);
                            let iff2 = cpu.regs.get_iff2();
                            let r = cpu.regs.get_r();
                            cpu.regs.set_acc(r);
                            let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
                            flags |= SZF3F5_TABLE[r as usize];
                            flags |= iff2 as u8 * FLAG_PV;
                            cpu.regs.set_flags(flags);
                        }
                        // RRD
                        U3::N4 => {
                            let mut acc = cpu.regs.get_acc();
                            let mut mem = bus.read(cpu.regs.get_hl(), 3);
                            // low nibble
                            let mem_lo = mem & 0x0F;
                            // mem_hi to mem_lo and clear hi nibble
                            mem = (mem >> 4) & 0x0F;
                            // acc_lo to mem_hi
                            mem |= (acc << 4) & 0xF0;
                            acc = (acc & 0xF0) | mem_lo;
                            cpu.regs.set_acc(acc);
                            bus.wait_loop(cpu.regs.get_hl(), 4);
                            bus.write(cpu.regs.get_hl(), mem, 3);
                            cpu.regs.set_mem_ptr(cpu.regs.get_hl().wrapping_add(1));
                            let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
                            flags |= SZPF3F5_TABLE[acc as usize];
                            cpu.regs.set_flags(flags);
                            // Clocks: 4 + 4 + 3 + 4 + 3 = 18
                        }
                        // RLD
                        U3::N5 => {
                            let mut acc = cpu.regs.get_acc();
                            let mut mem = bus.read(cpu.regs.get_hl(), 3);
                            // low nibble
                            let acc_lo = acc & 0x0F;
                            // mem_hi to acc_lo
                            acc = (acc & 0xF0) | ((mem >> 4) & 0x0F);
                            // mem_lo to mem_hi and tmp to mem_lo
                            mem = ((mem << 4) & 0xF0) | acc_lo;
                            cpu.regs.set_acc(acc);
                            bus.wait_loop(cpu.regs.get_hl(), 4);
                            bus.write(cpu.regs.get_hl(), mem, 3);
                            cpu.regs.set_mem_ptr(cpu.regs.get_hl().wrapping_add(1));
                            let mut flags = cpu.regs.get_flags() & FLAG_CARRY;
                            flags |= SZPF3F5_TABLE[acc as usize];
                            cpu.regs.set_flags(flags);
                            // Clocks: 4 + 4 + 3 + 4 + 3 = 18
                        }
                        // NOP
                        U3::N6 | U3::N7 => {
                            bus.process_unknown_opcode(Prefix::ED, opcode);
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
                                cpu.regs.set_mem_ptr(cpu.regs.get_pc().wrapping_sub(1));
                                // last DE for wait
                                bus.wait_loop(cpu.regs.get_de().wrapping_sub(1), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_mem_cycle();
                            };
                        }
                        // LDDR
                        U3::N7 => {
                            execute_ldi_ldd(cpu, bus, BlockDir::Dec);
                            if cpu.regs.get_reg_16(RegName16::BC) != 0 {
                                cpu.regs.set_mem_ptr(cpu.regs.get_pc().wrapping_sub(1));
                                // last DE for wait
                                bus.wait_loop(cpu.regs.get_de().wrapping_add(1), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_mem_cycle();
                            };
                        }
                        // NOP
                        _ => {
                            bus.process_unknown_opcode(Prefix::ED, opcode);
                        }
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
                                cpu.regs.set_mem_ptr(cpu.regs.get_pc().wrapping_sub(1));
                                // last HL
                                bus.wait_loop(cpu.regs.get_hl().wrapping_sub(1), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_mem_cycle();
                            };
                        }
                        // CPDR
                        U3::N7 => {
                            let result = execute_cpi_cpd(cpu, bus, BlockDir::Dec);
                            if (cpu.regs.get_reg_16(RegName16::BC) != 0) & (!result) {
                                cpu.regs.set_mem_ptr(cpu.regs.get_pc().wrapping_sub(1));
                                bus.wait_loop(cpu.regs.get_hl().wrapping_add(1), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_mem_cycle();
                            };
                        }
                        // NOP
                        _ => {
                            bus.process_unknown_opcode(Prefix::ED, opcode);
                        }
                    }
                }
                // IN Block group
                U3::N2 => {
                    match opcode.y {
                        // INI
                        U3::N4 => {
                            execute_ini_ind(cpu, bus, BlockDir::Inc);
                        }
                        // IND
                        U3::N5 => {
                            execute_ini_ind(cpu, bus, BlockDir::Dec);
                        }
                        // INIR
                        U3::N6 => {
                            let m = execute_ini_ind(cpu, bus, BlockDir::Inc);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                bus.wait_loop(cpu.regs.get_hl().wrapping_sub(1), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_io_cycle(BlockIoOpcode::Inir, m);
                            };
                        }
                        // INDR
                        U3::N7 => {
                            let m = execute_ini_ind(cpu, bus, BlockDir::Dec);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                bus.wait_loop(cpu.regs.get_hl().wrapping_add(1), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_io_cycle(BlockIoOpcode::Indr, m);
                            };
                        }
                        // NOP
                        _ => {
                            bus.process_unknown_opcode(Prefix::ED, opcode);
                        }
                    }
                }
                // Out Block group
                U3::N3 => {
                    match opcode.y {
                        // OUTI
                        U3::N4 => {
                            execute_outi_outd(cpu, bus, BlockDir::Inc);
                        }
                        // OUTD
                        U3::N5 => {
                            execute_outi_outd(cpu, bus, BlockDir::Dec);
                        }
                        // OTIR
                        U3::N6 => {
                            let m = execute_outi_outd(cpu, bus, BlockDir::Inc);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                bus.wait_loop(cpu.regs.get_bc(), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_io_cycle(BlockIoOpcode::Otir, m);
                            };
                        }
                        // OTDR
                        U3::N7 => {
                            let m = execute_outi_outd(cpu, bus, BlockDir::Dec);
                            if cpu.regs.get_reg_8(RegName8::B) != 0 {
                                bus.wait_loop(cpu.regs.get_bc(), 5);
                                cpu.regs.dec_pc();
                                cpu.regs.dec_pc();
                                cpu.regs.update_flags_block_io_cycle(BlockIoOpcode::Otdr, m);
                            };
                        }
                        // NOP
                        _ => {
                            bus.process_unknown_opcode(Prefix::ED, opcode);
                        }
                    }
                }
                // NOP
                _ => {
                    bus.process_unknown_opcode(Prefix::ED, opcode);
                }
            }
        }
    }
}
