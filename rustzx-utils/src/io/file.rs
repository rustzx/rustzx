use super::{into_std_seek_pos, DynamicAssetImpl, DynamicDataRecorderImpl};

use rustzx_core::{
    error::IoError,
    host::{DataRecorder, LoadableAsset, SeekFrom, SeekableAsset},
};

use std::{
    fs::File,
    io::{Read, Seek, Write},
};

pub struct FileAsset {
    file: File,
}

impl From<File> for FileAsset {
    fn from(file: File) -> Self {
        Self { file }
    }
}

impl SeekableAsset for FileAsset {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize, IoError> {
        self.file
            .seek(into_std_seek_pos(pos))
            .map_err(|e| {
                log::error!("Failed to seeek asset: {}", e);
                IoError::HostAssetImplFailed
            })
            .map(|count| count as usize)
    }
}

impl LoadableAsset for FileAsset {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        self.file.read(buf).map_err(|e| {
            log::error!("Failed to read asset: {}", e);
            IoError::HostAssetImplFailed
        })
    }
}

impl DataRecorder for FileAsset {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.file.write(buf).map_err(|e| {
            log::error!("Failed to write data to file: {}", e);
            IoError::HostAssetImplFailed
        })
    }
}

impl DynamicAssetImpl for FileAsset {}
impl DynamicDataRecorderImpl for FileAsset {}
