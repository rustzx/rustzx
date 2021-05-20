use displaydoc::Display;
use from_variants::FromVariants;

#[derive(Debug, Display, FromVariants)]
pub enum Error {
    /// Failed to read asset
    AssetRead(IoError),
    /// Failed to load rom
    RomLoad(RomLoadError),
    /// Failed to load rom
    TapeLoad(TapeLoadError),
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
