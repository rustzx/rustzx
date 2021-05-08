use alloc::{vec, vec::Vec};

// page size in bytes
pub const PAGE_SIZE: usize = 16 * 1024;
// different memory blocks size's
pub const SIZE_16K: usize = PAGE_SIZE;
pub const SIZE_32K: usize = PAGE_SIZE * 2;
pub const SIZE_48K: usize = PAGE_SIZE * 3;
pub const SIZE_64K: usize = PAGE_SIZE * 4;
pub const SIZE_128K: usize = PAGE_SIZE * 8;
// count of all memory blocks
pub const MEM_BLOCKS: usize = 4;

/// Rom can be:
/// - 16K (Sinclair48K)
/// - 32K (Sinclair128K, 2+)
/// - 64k (Amstrad 3+)
pub enum RomType {
    K16,
    K32,
    K64,
}

/// Ram can be:
/// - 48K (Sinclair48K)
/// - 128K (Sinclair128K, Amstrad 2+, Amstrad 3+)
pub enum RamType {
    K48,
    K128,
}

// Page info and type
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Ram(u8),
    Rom(u8),
}

// Memory struct
pub struct ZXMemory {
    rom: Vec<u8>,
    ram: Vec<u8>,
    // 4 x 16K blocks  map
    map: [Page; 4],
}

impl ZXMemory {
    /// Returns new Memory with coresponding rom and ram types
    pub fn new(rom_type: RomType, ram_type: RamType) -> ZXMemory {
        let ram_size;
        let mem_map;
        // build memory map.
        match ram_type {
            RamType::K48 => {
                ram_size = SIZE_48K;
                mem_map = [Page::Rom(0), Page::Ram(0), Page::Ram(1), Page::Ram(2)];
            }
            RamType::K128 => {
                ram_size = SIZE_128K;
                mem_map = [Page::Rom(0), Page::Ram(5), Page::Ram(2), Page::Ram(0)];
            }
        };
        let rom_size = match rom_type {
            RomType::K16 => SIZE_16K,
            RomType::K32 => SIZE_32K,
            RomType::K64 => SIZE_64K,
        };
        ZXMemory {
            rom: vec![0; rom_size],
            ram: vec![0; ram_size],
            map: mem_map,
        }
    }

    /// Returns value form memory
    pub fn read(&self, addr: u16) -> u8 {
        let page = self.map[(addr as usize) / PAGE_SIZE];
        let addr_rel = addr as usize % PAGE_SIZE;
        match page {
            Page::Rom(page) => self.rom[(page as usize) * PAGE_SIZE + addr_rel],
            Page::Ram(page) => self.ram[(page as usize) * PAGE_SIZE + addr_rel],
        }
    }

    /// Returns value of data with relative `addr` at `bank`
    pub fn paged_read(&self, page: Page, addr: u16) -> u8 {
        assert!((addr as usize) < PAGE_SIZE);
        match page {
            Page::Rom(page) => self.rom[(page as usize) * PAGE_SIZE + addr as usize],
            Page::Ram(page) => self.ram[(page as usize) * PAGE_SIZE + addr as usize],
        }
    }

    /// Writes value to memory
    pub fn write(&mut self, addr: u16, value: u8) {
        let page = self.map[(addr as usize) / PAGE_SIZE];
        let addr_rel = addr as usize % PAGE_SIZE;
        if let Page::Ram(page) = page {
            self.ram[(page as usize) * PAGE_SIZE + addr_rel] = value;
        }
    }

    /// Changes memory map
    /// # Panics
    /// panics when ram page number is out of range. This must me checked at
    /// development stage
    pub fn remap(&mut self, block: usize, page: Page) -> &mut ZXMemory {
        match page {
            Page::Ram(page) if (page as usize + 1) * PAGE_SIZE > self.ram.len() => {
                panic!("[ERROR] Ram page {} do not exists!", page);
            }
            Page::Rom(page) if (page as usize + 1) * PAGE_SIZE > self.rom.len() => {
                panic!("[ERROR] Rom page {} do not exists!", page);
            }
            _ => {}
        }
        self.map[block] = page;
        self
    }

    /// Returns bank type of mapped page
    pub fn get_bank_type(&self, block: usize) -> Page {
        assert!(block < MEM_BLOCKS);
        self.map[block]
    }

    /// Returns bank type of address
    pub fn get_page(&self, addr: u16) -> Page {
        self.map[addr as usize / PAGE_SIZE]
    }

    /// Returns mutable slice to rom page
    pub fn rom_page_data_mut(&mut self, page: u8) -> &mut [u8] {
        if (page as usize + 1) * PAGE_SIZE > self.rom.len() {
            panic!("[ERROR] Rom page {} does not exists!", page);
        }
        let shift = page as usize * PAGE_SIZE;
        &mut self.rom[shift..shift + PAGE_SIZE]
    }

    /// Returns mutable slice to ram page
    pub fn ram_page_data_mut(&mut self, page: u8) -> &mut [u8] {
        if (page as usize + 1) * PAGE_SIZE > self.ram.len() {
            panic!("[ERROR] Ram page {} does not exists!", page);
        }
        let shift = page as usize * PAGE_SIZE;
        &mut self.ram[shift..shift + PAGE_SIZE]
    }

    /// Dumps current address space
    pub fn dump(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for n in 0..SIZE_64K {
            out.push(self.read(n as u16));
        }
        out
    }
}
