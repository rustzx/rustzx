use crate::error::IoError;

type Result<T> = core::result::Result<T, IoError>;

#[derive(Clone, Copy)]
pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
}

/// Implementation of loadable asset for buffer-like type
pub struct BufferCursor<T: AsRef<[u8]>> {
    data: T,
    pos: usize,
}

impl<T: AsRef<[u8]>> BufferCursor<T> {
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T: AsRef<[u8]>> BufferCursor<T> {
    pub fn new(data: T) -> Self {
        Self { data, pos: 0 }
    }
}

impl<T: AsRef<[u8]>> SeekableAsset for BufferCursor<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos as isize,
            SeekFrom::End(pos) => self.data.as_ref().len() as isize + pos,
            SeekFrom::Current(pos) => self.pos as isize + pos,
        };
        if new_pos < 0 {
            return Err(IoError::SeekBeforeStart);
        }
        self.pos = new_pos as usize;

        Ok(self.pos)
    }
}

impl<T: AsRef<[u8]>> LoadableAsset for BufferCursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let data = self.data.as_ref();

        if self.pos >= data.len() {
            return Err(IoError::UnexpectedEof);
        }
        let bytes_to_read = buf.len().min(data.len() - self.pos);
        buf[0..bytes_to_read].copy_from_slice(&data[self.pos..self.pos + bytes_to_read]);
        self.pos += bytes_to_read;
        Ok(bytes_to_read)
    }
}

pub trait SeekableAsset {
    /// Seek position in the asset. Returns current position in the asset
    fn seek(&mut self, pos: SeekFrom) -> Result<usize>;
}

// While no_std environment does not provide Read
// trait, we are forced to use substitution for it
pub trait LoadableAsset {
    /// Read data from asset to `buf`
    /// Returns count of read bytes. Should return 0 read bytes when EOF was reached
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.read(buf)? {
                0 => {
                    break;
                }
                n => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                }
            }
        }
        if !buf.is_empty() {
            return Err(IoError::UnexpectedEof);
        }

        Ok(())
    }
}

pub trait DataRecorder {
    /// Writes given buffer to recorder.
    /// Returns count of written bytes or 0 if end of the
    /// destination asset was reached (e.g. buffer filled)
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Writes all bytes to the destination or returns
    /// [IoError::WriteZero] if destination refused to accept
    /// more bytes
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf)? {
                0 => return Err(IoError::WriteZero),
                n => buf = &buf[n..],
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_cursor_seek_works() {
        const BUFFER: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut cursor = BufferCursor::new(BUFFER);

        let mut tmp = [0u8; 1];
        cursor.read_exact(&mut tmp).unwrap();
        assert_eq!(tmp[0], 1);

        cursor.seek(SeekFrom::Current(1)).unwrap();
        cursor.read_exact(&mut tmp).unwrap();
        assert_eq!(tmp[0], 3);

        cursor.seek(SeekFrom::Start(2)).unwrap();
        cursor.read_exact(&mut tmp).unwrap();
        assert_eq!(tmp[0], 3);

        let mut tmp = [0u8; 3];
        cursor.seek(SeekFrom::End(-2)).unwrap();
        let read_bytes = cursor.read(&mut tmp).unwrap();
        assert_eq!(read_bytes, 2);
        assert_eq!(tmp[0], 9);
        assert_eq!(tmp[1], 10);
    }
}
