//! Contains all built-in ROM's.

/// Copyright (C) 1982 Sinclair Research Ltd. (now owned by Amstrad plc)
pub const ROM_48K: &'static [u8; 16 * 1024]  = include_bytes!("48.rom");
