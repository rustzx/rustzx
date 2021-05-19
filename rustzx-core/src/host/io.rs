use crate::error::IoError;

type Result<T> = core::result::Result<T, IoError>;

#[derive(Clone, Copy)]
pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
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

    /// Writes all bytes to the destiantion or returns
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
