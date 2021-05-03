use rustzx_core::{
    error::AssetReadError,
    host::{LoadableAsset, SeekFrom},
};

use std::{
    fs::File,
    io::{Read, Seek},
};

pub struct FileAsset {
    file: File,
}

impl From<File> for FileAsset {
    fn from(file: File) -> Self {
        Self { file }
    }
}

impl LoadableAsset for FileAsset {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, AssetReadError> {
        self.file.read(buf).map_err(|e| {
            log::error!("Failed to read asset: {}", e);
            AssetReadError::HostAssetImplFailed
        })
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<usize, AssetReadError> {
        self.file
            .seek(into_std_seek_pos(pos))
            .map_err(|e| {
                log::error!("Failed to seeek asset: {}", e);
                AssetReadError::HostAssetImplFailed
            })
            .map(|count| count as usize)
    }
}

fn into_std_seek_pos(pos: SeekFrom) -> std::io::SeekFrom {
    match pos {
        SeekFrom::Start(offset) => std::io::SeekFrom::Start(offset as u64),
        SeekFrom::End(offset) => std::io::SeekFrom::End(offset as i64),
        SeekFrom::Current(offset) => std::io::SeekFrom::Current(offset as i64),
    }
}
