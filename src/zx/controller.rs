//! Contains ZX Spectrum System contrller (like lua or so) of emulator
//! TODO: Make ZXController Builder

use std::fs::File;
use std::io::Read;

use utils::{split_word, Clocks};
use utils::screen::*;
use utils::events::*;
use utils::InstantFlag;
use z80::Z80Bus;
use zx::{ZXMemory, RomType, RamType};
use zx::memory::Page;
use zx::machine::ZXMachine;
use zx::tape::*;
use zx::ZXKey;
use zx::screen::canvas::ZXCanvas;
use zx::screen::border::ZXBorder;
use zx::screen::colors::{ZXColor, ZXPalette};
use zx::roms::*;

/// Tape loading trap at LD-BREAK routine in ROM
const ADDR_LD_BREAK: u16 = 0x056B;

/// ZX System controller
pub struct ZXController {
    pub machine: ZXMachine,
    pub memory: ZXMemory,
    pub canvas: ZXCanvas,
    pub tape: Box<ZXTape>,
    pub border: ZXBorder,
    keyboard: [u8; 8],
    border_color: u8,
    ear: bool,
    frame_clocks: Clocks,
    passed_frames: usize,
    events: EventQueue,
    instant_event: InstantFlag,
}

impl ZXController {
    /// Returns new ZXController
    pub fn new(machine: ZXMachine) -> ZXController {
        let memory = match machine {
            _ => ZXMemory::new(RomType::K16, RamType::K48),
        };
        let canvas = ZXCanvas::new(machine, ZXPalette::default());
        let border = ZXBorder::new(machine, ZXPalette::default());
        ZXController {
            machine: machine,
            memory: memory,
            canvas: canvas,
            border: border,
            keyboard: [0xFF; 8],
            border_color: 0x00,
            ear: true,
            frame_clocks: Clocks(0),
            passed_frames: 0,
            tape: Box::new(Tap::new()),
            events: EventQueue::new(),
            instant_event: InstantFlag::new(false),
        }
    }

    /// loads rom form file
    pub fn load_rom(&mut self, path: &str) {
        let mut rom = Vec::new();
        if let Ok(mut file) = File::open(path) {
            file.read_to_end(&mut rom).unwrap();
        } else {
            panic!("ROM not found!");
        }
        self.memory.load_rom(0, &rom).unwrap();
    }
    /// load builted-in ROM
    pub fn load_default_rom(&mut self) {
        self.memory.load_rom(0, ROM_48K).unwrap();
    }

    /// inserts new tape
    pub fn insert_tape(&mut self, path: &str) {
        self.tape.insert(path);
    }

    /// plays tape
    pub fn play_tape(&mut self) {
        self.tape.play();
    }

    /// stops tape
    pub fn stop_tape(&mut self) {
        self.tape.stop();
    }

    /// Returns Screen texture
    pub fn get_canvas_texture(&self) -> &[u8] {
        self.canvas.texture()
    }

    /// Returns border texture
    pub fn get_border_texture(&self) -> &[u8] {
        self.border.texture()
    }

    /// get current border color
    pub fn get_border_color(&self) -> u8 {
        self.border_color
    }

    /// Changes key state in controller
    pub fn send_key(&mut self, key: ZXKey, pressed: bool) {
        // TODO: Move row detection to ZXKey type
        let rownum = match key.half_port {
            0xFE => Some(0),
            0xFD => Some(1),
            0xFB => Some(2),
            0xF7 => Some(3),
            0xEF => Some(4),
            0xDF => Some(5),
            0xBF => Some(6),
            0x7F => Some(7),
            _ => None,
        };
        if let Some(rownum) = rownum {
            self.keyboard[rownum] = self.keyboard[rownum] & (!key.mask);
            if !pressed {
                self.keyboard[rownum] |= key.mask;
            }
        }
    }

    /// Dumps memory space
    pub fn dump(&self) -> Vec<u8> {
        self.memory.dump()
    }

    /// Returns current bus floating value
    fn floating_bus_value(&self) -> u8 {
        let specs = self.machine.specs();
        let clocks = self.frame_clocks;
        if clocks.count() < specs.clocks_first_pixel + 2 {
            return 0xFF;
        }
        let clocks = clocks.count() - (specs.clocks_first_pixel + 2);
        let row = clocks / specs.clocks_line;
        let clocks = clocks % specs.clocks_line;
        let col = (clocks / 8) * 2 + (clocks % 8) / 2;
        if row < 192 && clocks < 124 && ((clocks & 0x04) == 0) {
            if clocks % 2 == 0 {
                return self.memory.read(get_bitmap_line_addr(row) + col as u16);
            } else {
                let byte = (row / 8) * 32 + col;
                return self.memory.read(0x5800 + byte as u16);
            };
        }
        return 0xFF;
    }

    /// make contention
    fn do_contention(&mut self) {
        let contention = self.machine.contention_clocks(self.frame_clocks);
        self.wait_internal(contention);
    }

    ///make contention + wait some clocks
    fn do_contention_and_wait(&mut self, wait_time: Clocks) {
        let contention = self.machine.contention_clocks(self.frame_clocks);
        self.wait_internal(contention + wait_time);
    }

    /// Returns early IO contention clocks
    fn io_contention_first(&mut self, port: u16) {
        if self.machine.addr_is_contended(port) {
            self.do_contention();
        };
        self.wait_internal(Clocks(1));
    }

    /// Returns late IO contention clocks
    fn io_contention_last(&mut self, port: u16) {
        if self.machine.port_is_contended(port) {
            self.do_contention_and_wait(Clocks(2));
        } else {
            if self.machine.addr_is_contended(port) {
                self.do_contention_and_wait(Clocks(1));
                self.do_contention_and_wait(Clocks(1));
                self.do_contention();
            } else {
                self.wait_internal(Clocks(2));
            }
        }
    }

    /// Starts a new frame
    fn new_frame(&mut self) {
        self.frame_clocks -= self.machine.specs().clocks_frame;
        self.canvas.new_frame();
        self.border.new_frame();
    }

    /// Validates screen
    pub fn validate_screen(&mut self) {
        for addr in 0x4000..0x5800 {
            self.canvas.write_bitmap_byte(addr, Clocks(0), self.memory.read(addr));
        }
        for addr in 0x5800..0x5B00 {
            self.canvas.write_attr_byte(addr, Clocks(0), self.memory.read(addr));
        }
    }

    /// force clears all events
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// check events count
    pub fn no_events(&self) -> bool {
        self.events.is_empty()
    }

    /// Returns last event
    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.receive_event()
    }

    /// Returns true if all frame clocks has been passed
    pub fn frames_count(&self) -> usize {
        self.passed_frames
    }

    pub fn reset_frame_counter(&mut self) {
        self.passed_frames = 0;
    }

    /// Returns current clocks from frame start
    pub fn clocks(&self) -> Clocks {
        self.frame_clocks
    }
}

impl Z80Bus for ZXController {

    /// we need to check different breakpoints like tape
    /// loading detection breakpoint
    fn pc_callback(&mut self, addr: u16) {
        // check mapped memory page at 0x0000 .. 0x3FFF
        match self.memory.get_page_type(0) {
            // if page is 48K Rom
            Page::Rom(0) => {
                // Tape LOAD/VERIFY
                if addr == ADDR_LD_BREAK {
                    // Add event (Fast tape loading request) it must be executed
                    // by emulator immediately
                    self.events.send_event(Event::new(EventKind::FastTapeLoad, self.frame_clocks));
                    self.instant_event.set();
                }
            }
            _ => {}
        }
    }


    fn read_internal(&mut self, addr: u16) -> u8 {
        self.memory.read(addr)
    }

    fn write_internal(&mut self, addr: u16, data: u8) {
        self.memory.write(addr, data);
        match addr {
            0x4000...0x57FF => {
                self.canvas.write_bitmap_byte(addr, self.frame_clocks, data);
            }
            0x5800...0x5AFF => {
                self.canvas.write_attr_byte(addr, self.frame_clocks, data);
            }
            _ => {}
        }
    }

    fn wait_internal(&mut self, clk: Clocks) {
        self.frame_clocks += clk;
        (*self.tape).process_clocks(clk);
        let ear = (*self.tape).current_bit();
        self.ear = ear;
        if self.frame_clocks.count() >= self.machine.specs().clocks_frame {
            self.new_frame();
            self.passed_frames += 1;
        }
    }

    fn wait_mreq(&mut self, addr: u16, clk: Clocks) {
        match self.machine {
            ZXMachine::Sinclair48K => {
                // contention in low 16k RAM
                if self.machine.addr_is_contended(addr) {
                    self.do_contention();
                }
            }
            _ => {}
        }
        self.wait_internal(clk);
    }

    fn wait_no_mreq(&mut self, addr: u16, clk: Clocks) {
        // only for 48 K!
        self.wait_mreq(addr, clk);
    }

    fn read_io(&mut self, port: u16) -> u8 {
        // all contentions check
        self.io_contention_first(port);
        self.io_contention_last(port);
        // find out what we need to do
        let (h, _) = split_word(port);
        let output = if port & 0x0001 == 0 {
            let mut tmp: u8 = 0xFF;
            for n in 0..8 {
                // if bit of row reset
                if ((h >> n) & 0x01) == 0 {
                    tmp &= self.keyboard[n];
                }
            }
            // invert bit 6 if ear active;
            if self.ear {
                tmp ^= 0x40;
            }
            // 5 and 7 unused
            tmp
        } else {
            self.floating_bus_value()
        };
        // add one clock after operation
        self.wait_internal(Clocks(1));
        output
    }

    fn write_io(&mut self, port: u16, data: u8) {
        // first contention
        self.io_contention_first(port);
        // if port from lua
        if port & 0x0001 == 0 {
            self.border_color = data & 0x07;
            self.border.set_border(self.frame_clocks, ZXColor::from_bits(data & 0x07));
        }
        // last contention after byte write
        self.io_contention_last(port);
        // add one clock after operation
        self.wait_internal(Clocks(1));
    }

    fn read_interrupt(&mut self) -> u8 {
        0xFF
    }

    fn int_active(&self) -> bool {
        self.frame_clocks.count() % self.machine.specs().clocks_frame <
        self.machine.specs().interrupt_length
    }

    fn nmi_active(&self) -> bool {
        false
    }
    fn reti(&mut self) {}

    fn halt(&mut self, _: bool) {}

    /// check for instant events
    fn instant_event(&self) -> bool {
        self.instant_event.pick()
    }
}
