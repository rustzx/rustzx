mod io;

pub use io::{LoadableAsset, SeekFrom};

pub enum Snapshot<LoadableAssetImpl: LoadableAsset> {
    Sna(LoadableAssetImpl),
    // TODO(#55): Implement SLT snapshot format support
}

pub enum Tape<LoadableAssetImpl: LoadableAsset> {
    Tap(LoadableAssetImpl),
    // TODO(#56): Implement TZX tape format support
}

pub enum RomFormat {
    Binary16KPages,
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
