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

fn into_std_seek_pos(pos: SeekFrom) -> std::io::SeekFrom {
    match pos {
        SeekFrom::Start(offset) => std::io::SeekFrom::Start(offset as u64),
        SeekFrom::End(offset) => std::io::SeekFrom::End(offset as i64),
        SeekFrom::Current(offset) => std::io::SeekFrom::Current(offset as i64),
    }
}
