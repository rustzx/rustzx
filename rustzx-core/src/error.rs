use displaydoc::Display;
use from_variants::FromVariants;

#[derive(Debug, Display, FromVariants)]
pub enum Error {
    /// Failed to read asset
    AssetRead(IoError),
    /// Failed to load rom
    RomLoad(RomLoadError),
    /// Failed to load tape
    TapeLoad(TapeLoadError),
    /// Failed to load screen
    ScreenLoad(ScreenLoadError),
    /// Failed to load snapshot
    SnapshotLoad(SnapshotLoadError),
}

#[derive(Debug, Display)]
pub enum IoError {
    /// Unexpected end of file
    UnexpectedEof,
    /// Sink unexpectedly refused to write more bytes
    WriteZero,
    /// Seek operation was performed with offset before beginning of the asset
    SeekBeforeStart,
    /// Host-provided asset implementation failed
    HostAssetImplFailed,
}

#[derive(Debug, Display)]
pub enum RomLoadError {
    /// More assets required to load rom
    MoreAssetsRequired,
}

#[derive(Debug, Display)]
pub enum TapeLoadError {
    /// Provided tap file is invalid
    InvalidTapFile,
}

#[derive(Debug, Display)]
pub enum SnapshotLoadError {
    /// Provided SNA file is invalid
    InvalidSNAFile,
    /// Provided SZX file is invalid
    InvalidSZXFile,
    /// Machine required by snapshot isn't supported
    MachineNotSupported,
    /// Zlib not supported
    ZlibNotSupported,
}

#[derive(Debug, Display)]
pub enum ScreenLoadError {
    /// Provided scr file is invalid
    InvalidScrFile,
    /// Selected machine can't be used to load given screen file
    MachineNotSupported,
}
