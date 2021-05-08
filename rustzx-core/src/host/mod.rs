mod io;

pub use io::{LoadableAsset, SeekFrom};

pub enum Snapshot<LoadableAssetImpl: LoadableAsset> {
    Sna(LoadableAssetImpl),
    // TODO: Implement SLT format support
}

pub enum Tape<LoadableAssetImpl: LoadableAsset> {
    Tap(LoadableAssetImpl),
    // TODO: Implement TZX format support
}

pub enum RomFormat {
    Binary16KPages
}

pub trait RomSet {
    type Asset: LoadableAsset;

    fn format(&self) -> RomFormat;
    fn next_asset(&mut self) -> Option<Self::Asset>;
}

/// Represents set of required types for emulator implementation
/// based on `rustzx-core`.
pub trait Host {
    /// File-like type implementation for tape loading
    type TapeAsset: LoadableAsset;
    /// File-like type implementation for snapshot loading
    type SnapshotAsset: LoadableAsset;
    /// File-like type implementation for rom loading
    type RomSet: RomSet;
}
