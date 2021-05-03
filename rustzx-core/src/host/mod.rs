mod io;

pub use io::LoadableAsset;

use crate::settings::RustzxSettings;

pub enum Snapshot<LoadableAssetImpl: LoadableAsset> {
    Sna(LoadableAssetImpl),
    // TODO: Implement SLT format support
}

pub enum Tape<LoadableAssetImpl: LoadableAsset> {
    Tap(LoadableAssetImpl),
    // TODO: Implement TZX format support
}

pub trait Host {
    /// File-like type implementation for tape loading
    type TapeAssetImpl: LoadableAsset;
    /// File-like type implementation for snapshot loading
    type SnapshotAssetImpl: LoadableAsset;
    /// File-like type implementation for rom loading
    type RomAssetImpl: LoadableAsset;

    /// Get custom rom image file. If host returns `None`,
    /// emulator will load default rom for the machine.
    fn rom(&self, page: usize) -> Option<Self::RomAssetImpl>;
    /// Get snapshot file to load
    fn snapshot(&self) -> Option<Snapshot<Self::SnapshotAssetImpl>>;
    /// Get tape file to insert on emulator start
    fn tape(&self) -> Option<Tape<Self::TapeAssetImpl>>;
    /// Return general emulator settings
    fn settings(&self) -> &RustzxSettings;
}
