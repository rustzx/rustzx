mod frame_buffer;
mod io;

pub use core::time::Duration;
pub use frame_buffer::{FrameBuffer, FrameBufferSource};
pub use io::{BufferCursor, DataRecorder, LoadableAsset, SeekFrom, SeekableAsset};

pub trait Stopwatch {
    fn new() -> Self;
    fn measure(&self) -> Duration;
}

pub enum Snapshot<LoadableAssetImpl: LoadableAsset> {
    Sna(LoadableAssetImpl),
    Szx(LoadableAssetImpl),
    // TODO(#55): Implement SLT snapshot format support
}

pub enum SnapshotRecorder<DataRecorderImpl: DataRecorder> {
    Sna(DataRecorderImpl),
    Szx(DataRecorderImpl),
}

pub enum Tape<LoadableAssetImpl: LoadableAsset> {
    Tap(LoadableAssetImpl),
    // TODO(#56): Implement TZX tape format support
}

pub enum Screen<LoadableAssetImpl: LoadableAsset> {
    Scr(LoadableAssetImpl),
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

pub trait ScreenAsset: LoadableAsset + SeekableAsset {}
impl<T> ScreenAsset for T where T: LoadableAsset + SeekableAsset {}

pub trait SnapshotAsset: LoadableAsset + SeekableAsset {}
impl<T> SnapshotAsset for T where T: LoadableAsset + SeekableAsset {}

/// Allows to extend base rustzx-core functionality by providing
/// interface for user-defined IO ports handling
pub trait IoExtender {
    /// Write byte value to io extender
    fn write(&mut self, port: u16, data: u8);
    /// Read byte value from io extender
    fn read(&mut self, port: u16) -> u8;
    /// Return true if io externder can process
    /// incoming read/write operation for a
    /// given port
    fn extends_port(&self, port: u16) -> bool;
}

/// IO externder which does nothing
pub struct StubIoExtender;

impl IoExtender for StubIoExtender {
    fn write(&mut self, _: u16, _: u8) {}

    fn read(&mut self, _: u16) -> u8 {
        0
    }

    fn extends_port(&self, _: u16) -> bool {
        false
    }
}

/// Allows to externd RustZX emulator with custom debug logic
pub trait DebugInterface {
    /// Returns true if breakpoint at given address is set and emulation should be stopped
    fn check_pc_breakpoint(&mut self, addr: u16) -> bool;
}

/// Debug interface which does nothing
pub struct StubDebugInterface;

impl DebugInterface for StubDebugInterface {
    fn check_pc_breakpoint(&mut self, _addr: u16) -> bool {
        false
    }
}

/// Represents set of required types for emulator implementation
/// based on `rustzx-core`.
pub trait Host {
    /// Immutable `Context` implementation which is used to obtain host-specific
    /// context objects for host-defined emulator parts construction (e.g. FrameBuffers)
    type Context: HostContext<Self>;
    /// File-like type implementation for tape loading
    type TapeAsset: LoadableAsset + SeekableAsset;
    /// Frame buffer implementation
    type FrameBuffer: FrameBuffer;
    /// Type which should provide methods to measure time intervals
    type EmulationStopwatch: Stopwatch;
    /// RustZX debug port implementation
    type IoExtender: IoExtender;
    /// Debug interface logic (e.g. breakpoints)
    type DebugInterface: DebugInterface;
}
