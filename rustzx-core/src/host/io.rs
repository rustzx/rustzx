use crate::error;

type Result<T> = core::result::Result<T, error::AssetReadError>;

#[derive(Clone, Copy)]
pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
}

// While no_std environment does not provide Read
// trait, we are forced to use substitution for it
pub trait LoadableAsset {
    /// Read data from asset to `buf`
    /// Return count of read bytes. Should return 0 read bytes when EOF was reached
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    /// Seek position in the asset. Returns current position in the asset
    fn seek(&mut self, pos: SeekFrom) -> Result<usize>;

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
            return Err(error::AssetReadError::UnexpectedEof);
        }

        Ok(())
    }

    fn read_to_end(&mut self, buf: &mut alloc::vec::Vec<u8>) -> Result<()> {
        let mut buffer = [0u8; 1024];
        let mut read_bytes = self.read(&mut buffer)?;
        while read_bytes != 0 {
            buf.extend_from_slice(&buffer[0..read_bytes]);
            read_bytes = self.read(&mut buffer)?;
        }
        Ok(())
    }
}
