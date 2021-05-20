mod frame_buffer;
mod io;

pub use frame_buffer::{FrameBuffer, FrameBufferSource};
pub use io::{DataRecorder, LoadableAsset, SeekFrom, SeekableAsset};

pub use io::BufferCursor;

pub enum Snapshot<LoadableAssetImpl: LoadableAsset> {
    Sna(LoadableAssetImpl),
    // TODO(#55): Implement SLT snapshot format support
}

pub enum SnapshotRecorder<DataRecorderImpl: DataRecorder> {
    Sna(DataRecorderImpl),
}

pub enum Tape<LoadableAssetImpl: LoadableAsset> {
    Tap(LoadableAssetImpl),
    // TODO(#56): Implement TZX tape format support
}

pub enum RomFormat {
    Binary16KPages,
}

pub trait RomSet {
    type Asset: LoadableAsset;

    fn format(&self) -> RomFormat;
    fn next_asset(&mut self) -> Option<Self::Asset>;
}

pub trait HostContext<H: Host + ?Sized>: Sized {
    fn frame_buffer_context(&self) -> <H::FrameBuffer as FrameBuffer>::Context;
}

/// Represents set of required types for emulator implementation
/// based on `rustzx-core`.
pub trait Host {
    /// Immutable `Context` implementation which is used to obtain host-specific
    /// context objects for host-defined emulator parts construction (e.g. FrameBuffers)
    type Context: HostContext<Self>;
    /// File-like type implementation for tape loading
    type TapeAsset: LoadableAsset + SeekableAsset;
    /// File-like type implementation for rom loading
    type RomSet: RomSet;
    /// Frame buffer implementation
    type FrameBuffer: FrameBuffer;
}
