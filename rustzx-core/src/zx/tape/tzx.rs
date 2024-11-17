use crate::{
    error::TapeLoadError,
    host::{LoadableAsset, SeekFrom, SeekableAsset},
    zx::tape::TapeImpl,
    Result,
};
const STD_PILOT_LENGTH: usize = 2168;
const STD_PILOT_PULSES_HEADER: usize = 8063;
const STD_PILOT_PULSES_DATA: usize = 3223;
const STD_SYNC1_LENGTH: usize = 667;
const STD_SYNC2_LENGTH: usize = 735;
const STD_BIT_ONE_LENGTH: usize = 1710;
const STD_BIT_ZERO_LENGTH: usize = 855;
// 1000ms in Tstates
const STD_PAUSE_LENGTH: usize = 3_500_000;
const BUFFER_SIZE: usize = 128;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TapeState {
    Init,
    Stop,
    Play,
    Process,
    Pilot { pulses_left: usize },
    PureTone { pulses_left: usize },
    PulseSequence { pulses_left: usize },
    Sync,
    NextByte { is_direct_recording_sample: bool },
    NextBit { mask: u8 },
    NextDirectRecordingBit { mask: u8 },
    BitHalf { half_bit_delay: usize, mask: u8 },
    Pause,
    Silence { length: usize },
}

// Tzx block id's are in hex
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TzxBlockId {
    Unknown = 0x0,
    StandardSpeedData = 0x10,
    TurboSpeedData = 0x11,
    PureTone = 0x12,
    PulseSequence = 0x13,
    PureDataBlock = 0x14,
    DirectRecording = 0x15,
    PauseOrSilence = 0x20,
    GroupStart = 0x21,
    GroupEnd = 0x22,
    LoopEnd = 0x25,
    StopIf48k = 0x2a,
    TextDescription = 0x30,
    /* Unsupported blocks follow */
    C64RomTypeData = 0x16,   // Deprecated
    C64TurboTapeData = 0x17, // Deprecated
    CswRecording = 0x18,
    GeneralizedData = 0x19,
    JumpToBlock = 0x23,
    LoopStart = 0x24,
    CallSequence = 0x26,
    ReturnFromSequence = 0x27,
    SelectBlock = 0x28,
    SetSignalLevel = 0x2b,
    MessageBlock = 0x31,
    ArchiveInfo = 0x32,
    HardwareType = 0x33,
    EmulationInfo = 0x34, // Deprecated
    CustomInfo = 0x35,    // Deprecated
    Snapshot = 0x40,      // Deprecated
    Glue = 0x5a,
}

impl From<u8> for TzxBlockId {
    fn from(num: u8) -> Self {
        match num {
            0x10 => TzxBlockId::StandardSpeedData,
            0x11 => TzxBlockId::TurboSpeedData,
            0x12 => TzxBlockId::PureTone,
            0x13 => TzxBlockId::PulseSequence,
            0x14 => TzxBlockId::PureDataBlock,
            0x15 => TzxBlockId::DirectRecording,
            0x16 => TzxBlockId::C64RomTypeData,
            0x17 => TzxBlockId::C64TurboTapeData,
            0x18 => TzxBlockId::CswRecording,
            0x19 => TzxBlockId::GeneralizedData,
            0x20 => TzxBlockId::PauseOrSilence,
            0x21 => TzxBlockId::GroupStart,
            0x22 => TzxBlockId::GroupEnd,
            0x23 => TzxBlockId::JumpToBlock,
            0x24 => TzxBlockId::LoopStart,
            0x25 => TzxBlockId::LoopEnd,
            0x26 => TzxBlockId::CallSequence,
            0x27 => TzxBlockId::ReturnFromSequence,
            0x28 => TzxBlockId::SelectBlock,
            0x2a => TzxBlockId::StopIf48k,
            0x2b => TzxBlockId::SetSignalLevel,
            0x30 => TzxBlockId::TextDescription,
            0x31 => TzxBlockId::MessageBlock,
            0x32 => TzxBlockId::ArchiveInfo,
            0x33 => TzxBlockId::HardwareType,
            0x34 => TzxBlockId::EmulationInfo,
            0x35 => TzxBlockId::CustomInfo,
            0x40 => TzxBlockId::Snapshot,
            0x5a => TzxBlockId::Glue,
            _ => TzxBlockId::Unknown,
        }
    }
}

pub struct TapeTimings {
    pilot_length: usize,
    sync1_length: usize,
    sync2_length: usize,
    bit_0_length: usize,
    bit_1_length: usize,
    pilot_pulses_header: usize,
    pilot_pulses_data: usize,
    pause_length: usize,
    // Used by Turbo speed data block
    pilot_tone_length: Option<usize>,
}

impl TapeTimings {
    pub fn default() -> Self {
        Self {
            pilot_length: STD_PILOT_LENGTH,
            pilot_pulses_header: STD_PILOT_PULSES_HEADER,
            pilot_pulses_data: STD_PILOT_PULSES_DATA,
            sync1_length: STD_SYNC1_LENGTH,
            sync2_length: STD_SYNC2_LENGTH,
            bit_0_length: STD_BIT_ZERO_LENGTH,
            bit_1_length: STD_BIT_ONE_LENGTH,
            pause_length: STD_PAUSE_LENGTH,
            pilot_tone_length: None,
        }
    }
}

pub struct Tzx<A: LoadableAsset + SeekableAsset> {
    asset: A,
    state: TapeState,
    prev_state: TapeState,
    buffer: [u8; BUFFER_SIZE],
    buffer_offset: usize,
    block_bytes_read: usize,
    current_block_id: Option<TzxBlockId>,
    current_block_size: Option<usize>,
    tape_ended: bool,
    // Non-fastload related fields
    curr_bit: bool,
    curr_byte: u8,
    delay: isize,
    tape_timings: TapeTimings,
    used_bits_in_last_byte: usize,
    bits_to_process_in_byte: usize,
    loop_start_marker: usize,
    num_repetitions: Option<u16>,
    is_48k_mode: bool,
}

impl<A: LoadableAsset + SeekableAsset> Tzx<A> {
    pub fn from_asset(asset: A, is48k: bool) -> Result<Self> {
        let tzx = Self {
            prev_state: TapeState::Stop,
            state: TapeState::Init,
            curr_bit: false,
            curr_byte: 0x00,
            buffer: [0u8; BUFFER_SIZE],
            buffer_offset: 0,
            block_bytes_read: 0,
            current_block_id: None,
            current_block_size: None,
            delay: 0,
            asset,
            tape_ended: false,
            tape_timings: TapeTimings::default(),
            used_bits_in_last_byte: 8,
            bits_to_process_in_byte: 0,
            loop_start_marker: 0,
            num_repetitions: None,
            is_48k_mode: is48k,
        };
        Ok(tzx)
    }
}

impl<A: LoadableAsset + SeekableAsset> TapeImpl for Tzx<A> {
    fn can_fast_load(&self) -> bool {
        self.state == TapeState::Stop
    }

    fn next_block_byte(&mut self) -> Result<Option<u8>> {
        if self.tape_ended {
            return Ok(None);
        }

        if let Some(block_size) = self.current_block_size {
            if self.block_bytes_read >= block_size {
                return Ok(None);
            }

            let mut buffer_read_pos = self.block_bytes_read - self.buffer_offset;

            // Read new buffer if required
            if buffer_read_pos >= BUFFER_SIZE {
                let bytes_to_read =
                    (block_size - self.buffer_offset - BUFFER_SIZE).min(BUFFER_SIZE);
                self.asset.read_exact(&mut self.buffer[0..bytes_to_read])?;
                self.buffer_offset += BUFFER_SIZE;
                buffer_read_pos = 0;
            }

            // Check last byte in block
            if self.block_bytes_read >= block_size {
                self.current_block_size = None;
                self.block_bytes_read = 0;
                return Ok(None);
            }

            // Perform actual read and advance position
            let result = self.buffer[buffer_read_pos];
            self.block_bytes_read += 1;

            if self.block_bytes_read >= block_size {
                self.bits_to_process_in_byte = self.used_bits_in_last_byte;
            } else {
                self.bits_to_process_in_byte = 8;
            }
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn next_block(&mut self) -> Result<bool> {
        if self.tape_ended {
            return Ok(false);
        }

        let mut id_size_buffer = [0u8; 1];
        if self.asset.read_exact(&mut id_size_buffer).is_err() {
            self.tape_ended = true;
            return Ok(false);
        }

        let block_id = TzxBlockId::from(id_size_buffer[0]);
        self.buffer_offset = 0;
        self.block_bytes_read = 0;
        match block_id {
            TzxBlockId::StandardSpeedData => {
                let mut block_header = [0u8; 4];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.tape_timings.pilot_length = STD_PILOT_LENGTH;
                self.tape_timings.sync1_length = STD_SYNC1_LENGTH;
                self.tape_timings.sync2_length = STD_SYNC2_LENGTH;
                self.tape_timings.pilot_pulses_header = STD_PILOT_PULSES_HEADER;
                self.tape_timings.pilot_pulses_data = STD_PILOT_PULSES_DATA;
                self.tape_timings.bit_0_length = STD_BIT_ZERO_LENGTH;
                self.tape_timings.bit_1_length = STD_BIT_ONE_LENGTH;
                self.tape_timings.pause_length =
                    u16::from_le_bytes([block_header[0], block_header[1]]) as usize;
                let block_size = u16::from_le_bytes([block_header[2], block_header[3]]) as usize;
                let block_bytes_to_read = block_size.min(BUFFER_SIZE);
                self.asset
                    .read_exact(&mut self.buffer[0..block_bytes_to_read])?;
                self.current_block_id = Some(TzxBlockId::StandardSpeedData);
                self.current_block_size = Some(block_size);
            }
            TzxBlockId::TurboSpeedData => {
                let mut block_header = [0u8; 18];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.tape_timings.pilot_length =
                    u16::from_le_bytes([block_header[0], block_header[1]]) as usize;
                self.tape_timings.sync1_length =
                    u16::from_le_bytes([block_header[2], block_header[3]]) as usize;
                self.tape_timings.sync2_length =
                    u16::from_le_bytes([block_header[4], block_header[5]]) as usize;
                self.tape_timings.bit_0_length =
                    u16::from_le_bytes([block_header[6], block_header[7]]) as usize;
                self.tape_timings.bit_1_length =
                    u16::from_le_bytes([block_header[8], block_header[9]]) as usize;
                self.tape_timings.pilot_tone_length =
                    Some(u16::from_le_bytes([block_header[10], block_header[11]]) as usize);
                self.used_bits_in_last_byte = block_header[12] as usize;
                self.tape_timings.pause_length =
                    u16::from_le_bytes([block_header[13], block_header[14]]) as usize;
                let block_size =
                    u32::from_le_bytes([block_header[15], block_header[16], block_header[17], 0])
                        as usize;
                let block_bytes_to_read = block_size.min(BUFFER_SIZE);
                self.asset
                    .read_exact(&mut self.buffer[0..block_bytes_to_read])?;
                self.current_block_id = Some(TzxBlockId::TurboSpeedData);
                self.current_block_size = Some(block_size);
            }
            TzxBlockId::PureTone => {
                let mut block_header = [0u8; 4];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.tape_timings.pilot_length =
                    u16::from_le_bytes([block_header[0], block_header[1]]) as usize;
                self.tape_timings.pilot_tone_length =
                    Some(u16::from_le_bytes([block_header[2], block_header[3]]) as usize);

                self.current_block_id = Some(TzxBlockId::PureTone);
                return Ok(true);
            }
            TzxBlockId::PulseSequence => {
                let mut block_header = [0u8; 1];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.tape_timings.pilot_tone_length = Some(block_header[0] as usize);
                let block_size = (block_header[0] as usize) * 2;
                let block_bytes_to_read = block_size.min(BUFFER_SIZE);
                self.asset
                    .read_exact(&mut self.buffer[0..block_bytes_to_read])?;
                self.current_block_size = Some(block_size);
                self.current_block_id = Some(TzxBlockId::PulseSequence);
                return Ok(true);
            }
            TzxBlockId::PureDataBlock => {
                let mut block_header = [0u8; 10];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.tape_timings.bit_0_length =
                    u16::from_le_bytes([block_header[0], block_header[1]]) as usize;
                self.tape_timings.bit_1_length =
                    u16::from_le_bytes([block_header[2], block_header[3]]) as usize;
                self.tape_timings.pilot_tone_length = None;
                self.used_bits_in_last_byte = block_header[4] as usize;
                self.tape_timings.pause_length =
                    u16::from_le_bytes([block_header[5], block_header[6]]) as usize;
                let block_size =
                    u32::from_le_bytes([block_header[7], block_header[8], block_header[9], 0])
                        as usize;
                let block_bytes_to_read = block_size.min(BUFFER_SIZE);
                self.asset
                    .read_exact(&mut self.buffer[0..block_bytes_to_read])?;
                self.current_block_id = Some(TzxBlockId::PureDataBlock);
                self.current_block_size = Some(block_size);
            }
            TzxBlockId::DirectRecording => {
                let mut block_header = [0u8; 8];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.tape_timings.bit_0_length =
                    u16::from_le_bytes([block_header[0], block_header[1]]) as usize;
                self.tape_timings.pause_length =
                    u16::from_le_bytes([block_header[2], block_header[3]]) as usize;
                self.used_bits_in_last_byte = block_header[4] as usize;
                let block_size =
                    u32::from_le_bytes([block_header[5], block_header[6], block_header[7], 0])
                        as usize;
                let block_bytes_to_read = block_size.min(BUFFER_SIZE);

                if self
                    .asset
                    .read_exact(&mut self.buffer[0..block_bytes_to_read])
                    .is_err()
                {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.current_block_id = Some(TzxBlockId::DirectRecording);
                self.current_block_size = Some(block_size);
                return Ok(true);
            }
            TzxBlockId::C64RomTypeData
            | TzxBlockId::C64TurboTapeData
            | TzxBlockId::CswRecording
            | TzxBlockId::GeneralizedData
            | TzxBlockId::SetSignalLevel => {
                let mut block_header = [0u8; 4];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                let block_size = u32::from_le_bytes(block_header) as isize;
                self.asset.seek(SeekFrom::Current(block_size))?;
                self.current_block_id = Some(TzxBlockId::Unknown);
                return Ok(true);
            }
            TzxBlockId::EmulationInfo | TzxBlockId::CustomInfo | TzxBlockId::Snapshot => {}
            TzxBlockId::PauseOrSilence => {
                let mut block_header = [0u8; 2];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                let val = u16::from_le_bytes(block_header) as usize;
                // Stop tape
                if val == 0 {
                    self.delay = 0;
                    return Ok(false);
                }
                // Save the value to buffer for later read by pause block
                self.buffer[0] = block_header[0];
                self.buffer[1] = block_header[1];
                self.current_block_id = Some(TzxBlockId::PauseOrSilence);
                return Ok(true);
            }
            TzxBlockId::GroupStart => {
                let mut block_header = [0u8; 1];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                let num_chars = block_header[0];
                if self
                    .asset
                    .read_exact(&mut self.buffer[0..num_chars as usize])
                    .is_err()
                {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.current_block_id = Some(TzxBlockId::GroupStart);
                return Ok(true);
            }
            TzxBlockId::GroupEnd => {
                self.current_block_id = Some(TzxBlockId::GroupEnd);
                return Ok(true);
            }
            TzxBlockId::LoopStart => {
                let mut block_header = [0u8; 2];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.num_repetitions = Some(u16::from_le_bytes(block_header));
                self.loop_start_marker = self.asset.seek(SeekFrom::Current(0))?;
            }
            TzxBlockId::LoopEnd => {
                self.current_block_id = Some(TzxBlockId::LoopEnd);
                if let Some(mut num_rep) = self.num_repetitions {
                    if num_rep > 0 {
                        num_rep -= 1;
                        self.num_repetitions = Some(num_rep);
                        self.asset.seek(SeekFrom::Start(self.loop_start_marker))?;
                        return Ok(true);
                    }
                }
                self.num_repetitions = None;
                return Ok(true);
            }
            TzxBlockId::SelectBlock => {
                let mut block_header = [0u8; 2];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                let block_size = u16::from_le_bytes(block_header) as isize;
                self.asset.seek(SeekFrom::Current(block_size))?;
                self.current_block_id = Some(TzxBlockId::Unknown);
                return Ok(true);
            }
            TzxBlockId::StopIf48k => {
                let mut block_header = [0u8; 4];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                if self.is_48k_mode {
                    return Ok(false);
                }
                self.current_block_id = Some(TzxBlockId::StopIf48k);
                return Ok(true);
            }
            TzxBlockId::TextDescription => {
                let mut num_chars_header = [0u8; 1];
                if self.asset.read_exact(&mut num_chars_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                let num_chars = num_chars_header[0];
                if self
                    .asset
                    .read_exact(&mut self.buffer[0..num_chars as usize])
                    .is_err()
                {
                    self.tape_ended = true;
                    return Ok(false);
                }
                self.current_block_size = Some(num_chars as usize);
                self.current_block_id = Some(TzxBlockId::TextDescription);
                return Ok(true);
            }
            TzxBlockId::ArchiveInfo => {
                let mut block_header = [0u8; 2];
                if self.asset.read_exact(&mut block_header).is_err() {
                    self.tape_ended = true;
                    return Ok(false);
                }
                let block_size = u16::from_le_bytes(block_header) as isize;
                self.asset.seek(SeekFrom::Current(block_size))?;
                self.current_block_id = Some(TzxBlockId::Unknown);
                return Ok(true);
            }
            _ => {
                return Ok(true);
            }
        }

        Ok(true)
    }

    fn current_bit(&self) -> bool {
        self.curr_bit
    }

    fn process_clocks(&mut self, clocks: usize) -> Result<()> {
        if self.state == TapeState::Stop {
            return Ok(());
        }

        if self.delay > 0 {
            self.delay -= clocks as isize;
            if self.delay > 0 {
                return Ok(());
            }
        }

        'state_machine: loop {
            match self.state {
                TapeState::Init => {
                    const HEADER_SIZE: usize = 10;
                    // check if valid tzx
                    let mut header_size_buffer = [0u8; HEADER_SIZE];
                    self.asset.seek(SeekFrom::Start(0))?;
                    if self
                        .asset
                        .read_exact(&mut header_size_buffer[0..HEADER_SIZE])
                        .is_ok()
                    {
                        self.state = TapeState::Play;
                    } else {
                        return Err(TapeLoadError::InvalidTapFile.into());
                    }
                    self.buffer_offset += HEADER_SIZE;
                    break 'state_machine;
                }
                TapeState::Stop => {
                    // Reset tape but leave in Stopped state
                    self.state = TapeState::Stop;
                    break 'state_machine;
                }
                TapeState::Play => {
                    if !self.next_block()? {
                        self.state = TapeState::Stop;
                    } else {
                        self.state = TapeState::Process;
                    }
                }
                TapeState::Process => {
                    if self.process_current_block().is_ok() {
                        break 'state_machine;
                    }
                }
                TapeState::Pilot { mut pulses_left } => {
                    self.curr_bit = !self.curr_bit;
                    pulses_left -= 1;
                    if pulses_left == 0 {
                        self.delay += self.tape_timings.sync1_length as isize;
                        self.state = TapeState::Sync;
                    } else {
                        self.delay += self.tape_timings.pilot_length as isize;
                        self.state = TapeState::Pilot { pulses_left };
                    }
                    break 'state_machine;
                }
                TapeState::PureTone { mut pulses_left } => {
                    self.curr_bit = !self.curr_bit;
                    pulses_left -= 1;
                    if pulses_left == 0 {
                        self.state = TapeState::Play;
                    } else {
                        self.delay += self.tape_timings.pilot_length as isize;
                        self.state = TapeState::PureTone { pulses_left };
                    }
                    break 'state_machine;
                }
                TapeState::PulseSequence { mut pulses_left } => {
                    self.curr_bit = !self.curr_bit;
                    pulses_left -= 1;
                    if pulses_left == 0 {
                        self.state = TapeState::Play;
                    } else {
                        // Read 2 bytes for the pulse length
                        let byte1 = self
                            .next_block_byte()?
                            .ok_or(TapeLoadError::InvalidTzxFile)?;
                        let byte2 = self
                            .next_block_byte()?
                            .ok_or(TapeLoadError::InvalidTzxFile)?;

                        self.delay += u16::from_le_bytes([byte1, byte2]) as isize;
                        self.state = TapeState::PulseSequence { pulses_left };
                    }
                    break 'state_machine;
                }
                TapeState::Sync => {
                    self.curr_bit = !self.curr_bit;
                    self.delay += self.tape_timings.sync2_length as isize;
                    self.state = TapeState::NextBit { mask: 0x80 };
                    break 'state_machine;
                }
                TapeState::NextByte {
                    is_direct_recording_sample,
                } => {
                    self.state = if let Some(byte) = self.next_block_byte()? {
                        self.curr_byte = byte;
                        if is_direct_recording_sample {
                            TapeState::NextDirectRecordingBit { mask: 0x80 }
                        } else {
                            TapeState::NextBit { mask: 0x80 }
                        }
                    } else {
                        TapeState::Pause
                    }
                }
                TapeState::NextBit { mask } => {
                    self.curr_bit = !self.curr_bit;

                    if (self.curr_byte & mask) == 0 {
                        self.delay += self.tape_timings.bit_0_length as isize;
                        self.state = TapeState::BitHalf {
                            half_bit_delay: self.tape_timings.bit_0_length,
                            mask,
                        };
                    } else {
                        self.delay += self.tape_timings.bit_1_length as isize;

                        self.state = TapeState::BitHalf {
                            half_bit_delay: self.tape_timings.bit_1_length,
                            mask,
                        };
                    };

                    break 'state_machine;
                }
                // Direct Recording sample processing
                TapeState::NextDirectRecordingBit { mut mask } => {
                    let bit = self.curr_byte & mask == 0;
                    self.delay += self.tape_timings.bit_0_length as isize;

                    if bit != self.curr_bit {
                        self.curr_bit = !self.curr_bit;
                    }
                    mask >>= 1;
                    self.bits_to_process_in_byte -= 1;
                    self.state = if mask == 0 || self.bits_to_process_in_byte == 0 {
                        TapeState::NextByte {
                            is_direct_recording_sample: true,
                        }
                    } else {
                        TapeState::NextDirectRecordingBit { mask }
                    };
                    break 'state_machine;
                }
                TapeState::BitHalf {
                    half_bit_delay,
                    mut mask,
                } => {
                    self.curr_bit = !self.curr_bit;
                    self.delay += half_bit_delay as isize;
                    mask >>= 1;
                    self.bits_to_process_in_byte -= 1;

                    self.state = if mask == 0 || self.bits_to_process_in_byte == 0 {
                        TapeState::NextByte {
                            is_direct_recording_sample: false,
                        }
                    } else {
                        TapeState::NextBit { mask }
                    };
                    break 'state_machine;
                }

                TapeState::Pause => {
                    self.delay += (self.tape_timings.pause_length * 3_500) as isize;
                    self.state = TapeState::Play;
                    self.curr_bit = !self.curr_bit;
                    if self.delay > 0 {
                        break 'state_machine;
                    } // Next block or end of the tape
                }
                TapeState::Silence { length } => {
                    self.curr_bit = !self.curr_bit;
                    self.delay += (length * 3_500) as isize;
                    self.state = TapeState::Play;
                    break 'state_machine;
                }
            }
        }

        Ok(())
    }

    fn process_current_block(&mut self) -> Result<()> {
        if let Some(block_id) = &self.current_block_id.clone() {
            match block_id {
                TzxBlockId::StandardSpeedData => {
                    let first_byte = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;
                    // Select appropriate pulse count for Pilot sequence
                    let pulses_left = if first_byte == 0x00 {
                        self.tape_timings.pilot_pulses_header
                    } else {
                        self.tape_timings.pilot_pulses_data
                    };
                    self.curr_byte = first_byte;
                    self.delay += self.tape_timings.pilot_length as isize;
                    self.state = TapeState::Pilot { pulses_left };
                }
                TzxBlockId::TurboSpeedData => {
                    let first_byte = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;

                    // Select appropriate pulse count for Pilot sequence
                    let pulses_left = self.tape_timings.pilot_tone_length.unwrap();
                    self.curr_byte = first_byte;
                    self.delay += self.tape_timings.pilot_length as isize;
                    self.state = TapeState::Pilot { pulses_left };
                }
                TzxBlockId::DirectRecording => {
                    let first_byte = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;
                    self.curr_byte = first_byte;
                    self.curr_bit = !self.curr_bit;
                    self.state = TapeState::NextDirectRecordingBit { mask: 0x80 };
                }
                TzxBlockId::PureTone => {
                    let pulses_left = self.tape_timings.pilot_tone_length.unwrap();
                    self.delay += self.tape_timings.pilot_length as isize;
                    self.state = TapeState::PureTone { pulses_left };
                }
                TzxBlockId::PulseSequence => {
                    let pulses_left = self.tape_timings.pilot_tone_length.unwrap();

                    // Read 2 bytes for the pulse length
                    let byte1 = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;
                    let byte2 = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;

                    self.delay += u16::from_le_bytes([byte1, byte2]) as isize;
                    self.state = TapeState::PulseSequence { pulses_left };
                }
                TzxBlockId::PureDataBlock => {
                    let first_byte = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;
                    self.curr_byte = first_byte;
                    // Seem to need this flip for the block to load correctly.
                    self.curr_bit = !self.curr_bit;
                    self.state = TapeState::NextBit { mask: 0x80 };
                }
                TzxBlockId::PauseOrSilence => {
                    let byte1 = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;
                    let byte2 = self
                        .next_block_byte()?
                        .ok_or(TapeLoadError::InvalidTzxFile)?;

                    let length = u16::from_le_bytes([byte1, byte2]) as usize;
                    // Finish off previous edge first
                    self.delay += 3_500;
                    // Post that play "silence" for specified length
                    self.state = TapeState::Silence { length };
                }
                TzxBlockId::LoopEnd => {
                    self.delay = 0;
                    self.state = TapeState::Play;
                }
                TzxBlockId::GroupStart => {
                    self.delay = 0;
                    self.state = TapeState::Play;
                }
                TzxBlockId::GroupEnd => {
                    self.delay = 0;
                    self.state = TapeState::Play;
                }
                TzxBlockId::Unknown | TzxBlockId::StopIf48k => {
                    self.state = TapeState::Play;
                }
                _ => {
                    // Skip all bytes in the block
                    while self.next_block_byte()?.is_some() {}
                    self.state = TapeState::Play;
                }
            }
        }
        Ok(())
    }

    fn stop(&mut self) {
        let state = self.state;
        self.prev_state = state;
        self.state = TapeState::Stop;
    }

    fn play(&mut self) {
        if self.state == TapeState::Stop {
            if self.prev_state == TapeState::Stop {
                self.state = TapeState::Play;
            } else {
                self.state = self.prev_state;
            }
        }
    }

    fn rewind(&mut self) -> Result<()> {
        self.curr_bit = false;
        self.curr_byte = 0x00;
        self.block_bytes_read = 0;
        self.buffer_offset = 0;
        self.current_block_size = None;
        self.delay = 0;
        self.asset.seek(SeekFrom::Start(0))?;
        self.tape_ended = false;
        Ok(())
    }
}
