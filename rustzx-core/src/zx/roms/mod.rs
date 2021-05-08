//! Contains all built-in ROM's.

/// Copyright (C) 1982 Sinclair Research Ltd. (now owned by Amstrad plc)
pub const ROM_48K: &[u8; 16 * 1024] = include_bytes!("48.rom");
/// Copyright (C) 1986 Sinclair Research Ltd. (now owned by Amstrad plc)
pub const ROM_128K_0: &[u8; 16 * 1024] = include_bytes!("128.rom.0");
/// Copyright (C) 1982 Sinclair Research Ltd. (now owned by Amstrad plc)
pub const ROM_128K_1: &[u8; 16 * 1024] = include_bytes!("128.rom.1");
