//! Contains ZX Spectrum System contrller (like ula or so) of emulator
use crate::{
    error::Error,
    host::{Host, HostContext},
    settings::RustzxSettings,
    utils::screen::bitmap_line_addr,
    zx::{
        constants::{ADDR_LD_BREAK, CANVAS_HEIGHT, CLOCKS_PER_COL},
        events::EmulationEvents,
        joy::{
            kempston::KempstonJoy,
            sinclair::{self, SinclairJoyNum, SinclairKey},
        },
        keys::{CompoundKey, ZXKey},
        machine::ZXMachine,
        memory::{Page, RamType, RomType, ZXMemory, PAGE_SIZE},
        mouse::kempston::{KempstonMouse, KempstonMouseButton, KempstonMouseWheelDirection},
        tape::{TapeImpl, ZXTape},
        video::{colors::ZXColor, screen::ZXScreen},
    },
};
use rustzx_z80::Z80Bus;

#[cfg(feature = "embedded-roms")]
use crate::zx::roms;
#[cfg(feature = "sound")]
use crate::zx::sound::mixer::ZXMixer;
#[cfg(feature = "precise-border")]
use crate::zx::video::border::ZXBorder;

/// ZX System controller
pub(crate) struct ZXController<H: Host> {
    // parts of ZX Spectum.
    pub machine: ZXMachine,
    pub memory: ZXMemory,
    pub screen: ZXScreen<H::FrameBuffer>,
    pub tape: ZXTape<H::TapeAsset>,
    #[cfg(feature = "precise-border")]
    pub border: ZXBorder<H::FrameBuffer>,
    pub kempston: Option<KempstonJoy>,
    pub mouse: Option<KempstonMouse>,
    #[cfg(feature = "sound")]
    pub mixer: ZXMixer,
    pub keyboard: [u8; 8],
    pub keyboard_extended: [u8; 8],
    pub keyboard_sinclair: [u8; 8],
    pub caps_shift_modifier_mask: u32,
    // current border color
    pub border_color: ZXColor,
    // clocls count from frame start
    frame_clocks: usize,
    // frames count, which passed during emulation invokation
    passed_frames: usize,
    events: EmulationEvents,
    paging_enabled: bool,
    screen_bank: u8,
    current_port_7ffd: u8,
    // Z80 module expected controller implementation without errors,
    // so we need to store the internal errors manually. For sake of simplicity,
    // Only last error is saved
    last_emulation_error: Option<Error>,
}

impl<H: Host> ZXController<H> {
    /// Returns new ZXController from settings
    pub fn new(settings: &RustzxSettings, host_context: H::Context) -> Self {
        let (memory, paging, screen_bank);
        match settings.machine {
            ZXMachine::Sinclair48K => {
                memory = ZXMemory::new(RomType::K16, RamType::K48);
                paging = false;
                screen_bank = 0;
            }
            ZXMachine::Sinclair128K => {
                memory = ZXMemory::new(RomType::K32, RamType::K128);
                paging = true;
                screen_bank = 5;
            }
        };

        let kempston = if settings.kempston_enabled {
            Some(KempstonJoy::default())
        } else {
            None
        };

        let mouse = if settings.mouse_enabled {
            Some(KempstonMouse::default())
        } else {
            None
        };

        let screen = ZXScreen::new(settings.machine, host_context.frame_buffer_context());
        #[cfg(feature = "precise-border")]
        let border = ZXBorder::new(settings.machine, host_context.frame_buffer_context());

        #[cfg(feature = "sound")]
        let mixer = Self::create_mixer(settings);

        let out = ZXController {
            machine: settings.machine,
            memory,
            screen,
            #[cfg(feature = "precise-border")]
            border,
            kempston,
            mouse,
            #[cfg(feature = "sound")]
            mixer,
            keyboard: [0xFF; 8],
            keyboard_extended: [0xFF; 8],
            keyboard_sinclair: [0xFF; 8],
            caps_shift_modifier_mask: 0,
            border_color: ZXColor::Black,
            frame_clocks: 0,
            passed_frames: 0,
            tape: Default::default(),
            events: Default::default(),
            paging_enabled: paging,
            screen_bank,
            current_port_7ffd: 0,
            last_emulation_error: None,
        };

        #[cfg(feature = "embedded-roms")]
        if settings.load_default_rom {
            let mut out = out;
            out.load_default_rom();
            return out;
        }

        out
    }

    #[cfg(feature = "sound")]
    fn create_mixer(settings: &RustzxSettings) -> ZXMixer {
        let mut mixer = ZXMixer::new(
            settings.beeper_enabled,
            #[cfg(feature = "ay")]
            settings.ay_enabled,
            #[cfg(feature = "ay")]
            settings.ay_mode,
            settings.sound_sample_rate,
        );
        mixer.volume(settings.sound_volume as f64 / 200.0);
        mixer
    }

    /// returns current frame emulation pos in percents
    fn frame_pos(&self) -> f64 {
        let val = self.frame_clocks as f64 / self.machine.specs().clocks_frame as f64;
        if val > 1.0 {
            1.0
        } else {
            val
        }
    }

    /// loads builted-in ROM
    #[cfg(feature = "embedded-roms")]
    fn load_default_rom(&mut self) {
        match self.machine {
            ZXMachine::Sinclair48K => {
                let page = self.memory.rom_page_data_mut(0);
                page.copy_from_slice(roms::ROM_48K);
            }
            ZXMachine::Sinclair128K => {
                let page = self.memory.rom_page_data_mut(0);
                page.copy_from_slice(roms::ROM_128K_0);
                let page = self.memory.rom_page_data_mut(1);
                page.copy_from_slice(roms::ROM_128K_1);
            }
        }
    }

    /// Changes key state in controller
    pub fn send_key(&mut self, key: ZXKey, pressed: bool) {
        if pressed {
            self.keyboard[key.row_id()] &= !key.mask();
            return;
        }
        self.keyboard[key.row_id()] |= key.mask();
    }

    pub fn send_sinclair_key(&mut self, num: SinclairJoyNum, key: SinclairKey, pressed: bool) {
        let key = sinclair::sinclair_event_to_zx_key(key, num);
        if pressed {
            self.keyboard_sinclair[key.row_id()] &= !key.mask();
            return;
        }
        self.keyboard_sinclair[key.row_id()] |= key.mask();
    }

    pub fn send_compound_key(&mut self, key: CompoundKey, pressed: bool) {
        let mut dummy_modifier_mask = 0;
        let modifier_mask = match key.modifier_key() {
            ZXKey::Shift => &mut self.caps_shift_modifier_mask,
            _ => &mut dummy_modifier_mask,
        };
        let primary_key = key.primary_key();
        let modifier_key = key.modifier_key();

        if pressed {
            *modifier_mask |= key.modifier_mask();
            self.keyboard_extended[primary_key.row_id()] &= !primary_key.mask();
            self.keyboard_extended[modifier_key.row_id()] &= !modifier_key.mask();
        } else {
            *modifier_mask &= !key.modifier_mask();
            if *modifier_mask == 0 {
                self.keyboard_extended[modifier_key.row_id()] |= modifier_key.mask();
            }
            self.keyboard_extended[primary_key.row_id()] |= primary_key.mask();
        }
    }

    pub fn send_mouse_button(&mut self, button: KempstonMouseButton, pressed: bool) {
        if let Some(mouse) = &mut self.mouse {
            mouse.send_button(button, pressed);
        }
    }

    pub fn send_mouse_wheel(&mut self, dir: KempstonMouseWheelDirection) {
        if let Some(mouse) = &mut self.mouse {
            mouse.send_wheel(dir);
        }
    }

    pub fn send_mouse_pos_diff(&mut self, x: i8, y: i8) {
        if let Some(mouse) = &mut self.mouse {
            mouse.send_pos_diff(x, y);
        }
    }

    /// Returns current bus floating value
    fn floating_bus_value(&self) -> u8 {
        let specs = self.machine.specs();
        let clocks = self.frame_clocks;
        if clocks < specs.clocks_first_pixel + 2 {
            return 0xFF;
        }
        let clocks = clocks - (specs.clocks_first_pixel + 2);
        let row = clocks / specs.clocks_line;
        let clocks = clocks % specs.clocks_line;
        let col = (clocks / 8) * 2 + (clocks % 8) / 2;
        if row < CANVAS_HEIGHT
            && clocks < specs.clocks_screen_row - CLOCKS_PER_COL
            && ((clocks & 0x04) == 0)
        {
            if clocks % 2 == 0 {
                return self.memory.read(bitmap_line_addr(row) + col as u16);
            } else {
                let byte = (row / 8) * 32 + col;
                return self.memory.read(0x5800 + byte as u16);
            };
        }
        0xFF
    }

    /// make contention
    fn do_contention(&mut self) {
        let contention = self.machine.contention_clocks(self.frame_clocks);
        self.wait_internal(contention);
    }

    /// make contention + wait some clocks
    fn do_contention_and_wait(&mut self, wait_time: usize) {
        let contention = self.machine.contention_clocks(self.frame_clocks);
        self.wait_internal(contention + wait_time);
    }

    // check addr contention
    fn addr_is_contended(&self, addr: u16) -> bool {
        if let Page::Ram(bank) = self.memory.get_page(addr) {
            self.machine.bank_is_contended(bank as usize)
        } else {
            false
        }
    }

    /// Returns early IO contention clocks
    fn io_contention_first(&mut self, port: u16) {
        if self.addr_is_contended(port) {
            self.do_contention();
        };
        self.wait_internal(1);
    }

    /// Returns late IO contention clocks
    fn io_contention_last(&mut self, port: u16) {
        if self.machine.port_is_contended(port) {
            self.do_contention_and_wait(2);
        } else if self.addr_is_contended(port) {
            self.do_contention_and_wait(1);
            self.do_contention_and_wait(1);
            self.do_contention();
        } else {
            self.wait_internal(2);
        }
    }

    /// Starts a new frame
    fn new_frame(&mut self) {
        self.frame_clocks -= self.machine.specs().clocks_frame;
        self.screen.new_frame();
        #[cfg(feature = "precise-border")]
        self.border.new_frame();
        #[cfg(feature = "sound")]
        self.mixer.new_frame();
    }

    /// Clears all detected
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Returns last events
    pub fn events(&self) -> EmulationEvents {
        self.events
    }

    /// Returns true if all frame clocks has been passed
    pub fn frames_count(&self) -> usize {
        self.passed_frames
    }

    pub fn reset_frame_counter(&mut self) {
        self.passed_frames = 0;
    }

    /// Returns current clocks from frame start
    pub fn clocks(&self) -> usize {
        self.frame_clocks
    }

    pub fn write_7ffd(&mut self, val: u8) {
        if !self.paging_enabled {
            return;
        }
        self.current_port_7ffd = val;
        // remap top 16K of the ram
        self.memory.remap(3, Page::Ram(val & 0x07));
        // third block is not pageable
        // second block is screen buffer, not pageable. but we need to change active buffer
        let new_screen_bank = if val & 0x08 == 0 { 5 } else { 7 };
        self.screen.switch_bank(new_screen_bank as usize);
        self.screen_bank = new_screen_bank;
        // remap ROM
        self.memory.remap(0, Page::Rom((val >> 4) & 0x01));
        // check paging allow bit
        if val & 0x20 != 0 {
            self.paging_enabled = false;
        }
    }

    pub fn read_7ffd(&self) -> u8 {
        self.current_port_7ffd
    }

    #[cfg(all(feature = "sound", feature = "ay"))]
    fn read_ay_port(&mut self) -> u8 {
        self.mixer.ay.read()
    }

    #[cfg(not(all(feature = "sound", feature = "ay")))]
    fn read_ay_port(&mut self) -> u8 {
        self.floating_bus_value()
    }

    #[cfg(all(feature = "sound", feature = "ay"))]
    fn write_ay_port(&mut self, value: u8) {
        self.mixer.ay.write(value);
    }

    #[cfg(not(all(feature = "sound", feature = "ay")))]
    fn write_ay_port(&mut self, _: u8) {}

    #[cfg(all(feature = "sound", feature = "ay"))]
    fn select_ay_reg(&mut self, value: u8) {
        self.mixer.ay.select_reg(value)
    }

    #[cfg(not(all(feature = "sound", feature = "ay")))]
    fn select_ay_reg(&mut self, _: u8) {}

    pub(crate) fn set_border_color(
        &mut self,
        #[allow(unused_variables)] clocks: usize,
        color: ZXColor,
    ) {
        self.border_color = color;
        #[cfg(feature = "precise-border")]
        self.border.set_border(clocks, color);
    }

    pub(crate) fn take_last_emulation_error(&mut self) -> Option<Error> {
        self.last_emulation_error.take()
    }

    pub(crate) fn refresh_memory_dependent_devices(&mut self) {
        match self.machine {
            ZXMachine::Sinclair48K => {
                for (idx, data) in self.memory.ram_page_data(0).iter().enumerate() {
                    self.screen.update(idx as u16, 0, *data);
                }
            }
            ZXMachine::Sinclair128K => {
                for (idx, data) in self.memory.ram_page_data(5).iter().enumerate() {
                    self.screen.update(idx as u16, 5, *data);
                }
                for (idx, data) in self.memory.ram_page_data(7).iter().enumerate() {
                    self.screen.update(idx as u16, 7, *data);
                }
            }
        }
    }
}

impl<H: Host> Z80Bus for ZXController<H> {
    /// we need to check different breakpoints like tape
    /// loading detection breakpoint
    fn pc_callback(&mut self, addr: u16) {
        // check mapped memory page at 0x0000 .. 0x3FFF
        let check_fast_load = match self.machine {
            ZXMachine::Sinclair48K if self.memory.get_bank_type(0) == Page::Rom(0) => true,
            ZXMachine::Sinclair128K if self.memory.get_bank_type(0) == Page::Rom(1) => true,
            _ => false,
        };
        if check_fast_load {
            // Tape LOAD/VERIFY
            if addr == ADDR_LD_BREAK {
                // Add event (Fast tape loading request) it must be executed
                // by emulator immediately
                self.events |= EmulationEvents::TAPE_FAST_LOAD_TRIGGER_DETECTED;
            }
        }
    }

    /// read data without taking onto account contention
    fn read_internal(&mut self, addr: u16) -> u8 {
        self.memory.read(addr)
    }

    /// write data without taking onto account contention
    fn write_internal(&mut self, addr: u16, data: u8) {
        self.memory.write(addr, data);
        // if ram then compare bank to screen bank
        if let Page::Ram(bank) = self.memory.get_page(addr) {
            self.screen
                .update(addr % PAGE_SIZE as u16, bank as usize, data);
        }
    }

    /// Cahnges internal state on clocks count change (emualtion processing)
    fn wait_internal(&mut self, clk: usize) {
        self.frame_clocks += clk;
        if let Err(e) = self.tape.process_clocks(clk) {
            self.last_emulation_error = Some(e);
        }
        #[cfg(feature = "sound")]
        {
            let pos = self.frame_pos();
            self.mixer.process(pos);
        }
        self.screen.process_clocks(self.frame_clocks);
        if self.frame_clocks >= self.machine.specs().clocks_frame {
            self.new_frame();
            self.passed_frames += 1;
        }
    }

    // wait with memory request pin active
    fn wait_mreq(&mut self, addr: u16, clk: usize) {
        match self.machine {
            ZXMachine::Sinclair48K | ZXMachine::Sinclair128K => {
                // contention in low 16k RAM
                if self.addr_is_contended(addr) {
                    self.do_contention();
                }
            }
        }
        self.wait_internal(clk);
    }

    /// wait without memory request pin active
    fn wait_no_mreq(&mut self, addr: u16, clk: usize) {
        // only for 48 K!
        self.wait_mreq(addr, clk);
    }

    /// read io from hardware
    fn read_io(&mut self, port: u16) -> u8 {
        // all contentions check
        self.io_contention_first(port);
        self.io_contention_last(port);
        // find out what we need to do
        let [_, h] = port.to_le_bytes();
        let output = if port & 0x0001 == 0 {
            // ULA port
            let mut tmp: u8 = 0xFF;
            for n in 0..8 {
                // if bit of row reset
                if ((h >> n) & 0x01) == 0 {
                    let keyboard_byte =
                        self.keyboard[n] & self.keyboard_extended[n] & self.keyboard_sinclair[n];
                    tmp &= keyboard_byte;
                }
            }

            // Emulate zx spectrum "issue 2" model.
            // For future "issue 3" implementation condition will be `!self.ear`, but
            // different zx spectrum "issues" emulation is not planned yet
            if !self.tape.current_bit() {
                tmp ^= 0x40;
            }
            // 5 and 7 bits are unused
            tmp
        } else if self.mouse.is_some() && (port & 0x0121 == 0x0001) {
            self.mouse.as_ref().unwrap().buttons_port
        } else if self.mouse.is_some() && (port & 0x0521 == 0x0101) {
            self.mouse.as_ref().unwrap().x_pos_port
        } else if self.mouse.is_some() && (port & 0x0521 == 0x0501) {
            self.mouse.as_ref().unwrap().y_pos_port
        } else if port & 0xC002 == 0xC000 {
            self.read_ay_port()
        } else if self.kempston.is_some() && (port & 0x00E0 == 0) {
            self.kempston.as_ref().unwrap().read()
        } else {
            self.floating_bus_value()
        };
        // add one clock after operation
        self.wait_internal(1);
        output
    }

    /// write value to hardware port
    fn write_io(&mut self, port: u16, data: u8) {
        // first contention
        self.io_contention_first(port);
        // find active port
        if port & 0xC002 == 0xC000 {
            self.select_ay_reg(data);
        } else if port & 0xC002 == 0x8000 {
            self.write_ay_port(data);
        } else if port & 0x0001 == 0 {
            self.set_border_color(self.frame_clocks, ZXColor::from_bits(data & 0x07));
            let mic = data & 0x08 != 0;
            let ear = data & 0x10 != 0;
            #[cfg(feature = "sound")]
            self.mixer.beeper.change_state(ear, mic);
        } else if (port & 0x8002 == 0) && (self.machine == ZXMachine::Sinclair128K) {
            self.write_7ffd(data);
        }
        // last contention after byte write
        self.io_contention_last(port);
        // add one clock after operation
        self.wait_internal(1);
    }

    /// value, requested during `INT0` interrupt
    fn read_interrupt(&mut self) -> u8 {
        0xFF
    }

    /// checks system maskable interrupt pin state
    fn int_active(&self) -> bool {
        self.frame_clocks % self.machine.specs().clocks_frame
            < self.machine.specs().interrupt_length
    }

    /// checks non-maskable interrupt pin state
    fn nmi_active(&self) -> bool {
        false
    }

    /// CPU calls it when RETI instruction was processed
    fn reti(&mut self) {}

    /// CPU calls when was being halted
    fn halt(&mut self, _: bool) {}
}
