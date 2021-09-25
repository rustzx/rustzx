use super::DynamicAssetImpl;

use rustzx_core::{
    error::IoError,
    host::{BufferCursor, LoadableAsset, SeekFrom, SeekableAsset},
};

use std::{
    io::{self, Read},
    vec,
    vec::Vec,
};

use flate2::read::GzDecoder;

pub struct GzipAsset {
    buffer: BufferCursor<Vec<u8>>,
}

impl GzipAsset {
    pub fn new(file: impl Read) -> Result<Self, io::Error> {
        // ZX Spectrum assets are small enough to use RAM for unpacked data
        let mut buffer = vec![];
        let _ = GzDecoder::new(file).read_to_end(&mut buffer)?;
        Ok(Self {
            buffer: BufferCursor::new(buffer),
        })
    }
}

impl SeekableAsset for GzipAsset {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, IoError> {
        self.buffer.seek(pos)
    }
}

impl LoadableAsset for GzipAsset {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        self.buffer.read(buf)
    }
}

impl DynamicAssetImpl for GzipAsset {}
