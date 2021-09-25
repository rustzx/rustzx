mod file;
mod gzip;

use rustzx_core::{
    error::IoError,
    host::{BufferCursor, DataRecorder, LoadableAsset, SeekFrom, SeekableAsset},
};

use std::boxed::Box;

pub use file::FileAsset;
pub use gzip::GzipAsset;

pub trait DynamicAssetImpl: LoadableAsset + SeekableAsset {}

impl<T: AsRef<[u8]>> DynamicAssetImpl for BufferCursor<T> {}

pub struct DynamicAsset {
    inner: Box<dyn DynamicAssetImpl>,
}

impl<T: DynamicAssetImpl + 'static> From<T> for DynamicAsset {
    fn from(inner: T) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl LoadableAsset for DynamicAsset {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        self.inner.read(buf)
    }
}

impl SeekableAsset for DynamicAsset {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, IoError> {
        self.inner.seek(pos)
    }
}

pub trait DynamicDataRecorderImpl: DataRecorder {}

pub struct DynamicDataRecorder {
    inner: Box<dyn DynamicDataRecorderImpl>,
}

impl DataRecorder for DynamicDataRecorder {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.inner.write(buf)
    }
}

pub(crate) fn into_std_seek_pos(pos: SeekFrom) -> std::io::SeekFrom {
    match pos {
        SeekFrom::Start(offset) => std::io::SeekFrom::Start(offset as u64),
        SeekFrom::End(offset) => std::io::SeekFrom::End(offset as i64),
        SeekFrom::Current(offset) => std::io::SeekFrom::Current(offset as i64),
    }
}
