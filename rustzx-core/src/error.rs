use displaydoc::Display;
use from_variants::FromVariants;

#[derive(Debug, Display, FromVariants)]
pub enum Error {
    /// Failed to read asset
    AssetReadError(AssetReadError),
}

#[derive(Debug, Display)]
pub enum AssetReadError {
    /// Unexpected end of file
    UnexpectedEof,
}
