use displaydoc::Display;
use from_variants::FromVariants;

#[derive(Debug, Display, FromVariants)]
pub enum Error {
    /// Failed to read asset
    AssetReadError(AssetReadError),
    /// Failed to load rom
    RomLoad(RomLoadError),
}

#[derive(Debug, Display)]
pub enum AssetReadError {
    /// Unexpected end of file
    UnexpectedEof,
    /// Host-provided asset implementation failed
    HostAssetImplFailed,
}

#[derive(Debug, Display)]
pub enum RomLoadError {
    /// More assets required to load rom
    MoreAssetsRequired,
}
