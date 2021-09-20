use crate::{
    error::TapeLoadError,
    host::{LoadableAsset, SeekFrom, SeekableAsset},
    zx::tape::TapeImpl,
    Result,
};

const PILOT_LENGTH: usize = 2168;
const PILOT_PULSES_HEADER: usize = 8063;
const PILOT_PULSES_DATA: usize = 3223;
const SYNC1_LENGTH: usize = 667;
const SYNC2_LENGTH: usize = 735;
const BIT_ONE_LENGTH: usize = 1710;
const BIT_ZERO_LENGTH: usize = 855;
const PAUSE_LENGTH: usize = 3_500_000;
const BUFFER_SIZE: usize = 128;

#[derive(PartialEq, Eq, Clone, Copy)]
enum TapeState {
    Stop,
    Play,
    Pilot { pulses_left: usize },
    Sync,
    NextByte,
    NextBit { mask: u8 },
    BitHalf { half_bit_delay: usize, mask: u8 },
    Pause,
}

pub struct Tap<A: LoadableAsset + SeekableAsset> {
    asset: A,
    state: TapeState,
    prev_state: TapeState,
    buffer: [u8; BUFFER_SIZE],
    bufer_offset: usize,
    block_bytes_read: usize,
    current_block_size: Option<usize>,
    tape_ended: bool,
    // Non-fastload related fields
    curr_bit: bool,
    curr_byte: u8,
    delay: usize,
}

impl<A: LoadableAsset + SeekableAsset> Tap<A> {
    pub fn from_asset(asset: A) -> Result<Self> {
        let tap = Self {
            prev_state: TapeState::Stop,
            state: TapeState::Stop,
            curr_bit: false,
            curr_byte: 0x00,
            buffer: [0u8; BUFFER_SIZE],
            bufer_offset: 0,
            block_bytes_read: 0,
            current_block_size: None,
            delay: 0,
            asset,
            tape_ended: false,
        };
        Ok(tap)
    }
}

impl<A: LoadableAsset + SeekableAsset> TapeImpl for Tap<A> {
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

            let mut buffer_read_pos = self.block_bytes_read - self.bufer_offset;

            // Read new buffer if required
            if buffer_read_pos >= BUFFER_SIZE {
                let bytes_to_read = (block_size - self.bufer_offset - BUFFER_SIZE).min(BUFFER_SIZE);
                self.asset.read_exact(&mut self.buffer[0..bytes_to_read])?;
                self.bufer_offset += BUFFER_SIZE;
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
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn next_block(&mut self) -> Result<bool> {
        if self.tape_ended {
            return Ok(false);
        }

        // Skip leftovers from the previous block
        while self.next_block_byte()?.is_some() {}

        let mut block_size_buffer = [0u8; 2];
        if self.asset.read_exact(&mut block_size_buffer).is_err() {
            self.tape_ended = true;
            return Ok(false);
        }
        let block_size = u16::from_le_bytes(block_size_buffer) as usize;
        let block_bytes_to_read = block_size.min(BUFFER_SIZE);
        self.asset
            .read_exact(&mut self.buffer[0..block_bytes_to_read])?;

        self.bufer_offset = 0;
        self.block_bytes_read = 0;
        self.current_block_size = Some(block_size);

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
            if clocks > self.delay {
                self.delay = 0;
            } else {
                self.delay -= clocks;
            }
            return Ok(());
        }

        'state_machine: loop {
            match self.state {
                TapeState::Stop => {
                    // Reset tape but leave in Stopped state
                    self.rewind()?;
                    self.state = TapeState::Stop;
                    break 'state_machine;
                }
                TapeState::Play => {
                    if !self.next_block()? {
                        self.state = TapeState::Stop;
                    } else {
                        let first_byte = self
                            .next_block_byte()?
                            .ok_or(TapeLoadError::InvalidTapFile)?;

                        // Select appropriate pulse count for Pilot sequence
                        let pulses_left = if first_byte == 0x00 {
                            PILOT_PULSES_HEADER
                        } else {
                            PILOT_PULSES_DATA
                        };
                        self.curr_byte = first_byte;
                        self.curr_bit = true;
                        self.delay = PILOT_LENGTH;
                        self.state = TapeState::Pilot { pulses_left };
                        break 'state_machine;
                    }
                }
                TapeState::Pilot { mut pulses_left } => {
                    self.curr_bit = !self.curr_bit;
                    pulses_left -= 1;
                    if pulses_left == 0 {
                        self.delay = SYNC1_LENGTH;
                        self.state = TapeState::Sync;
                    } else {
                        self.delay = PILOT_LENGTH;
                        self.state = TapeState::Pilot { pulses_left };
                    }
                    break 'state_machine;
                }
                TapeState::Sync => {
                    self.curr_bit = !self.curr_bit;
                    self.delay = SYNC2_LENGTH;
                    self.state = TapeState::NextBit { mask: 0x80 };
                    break 'state_machine;
                }
                TapeState::NextByte => {
                    self.state = if let Some(byte) = self.next_block_byte()? {
                        self.curr_byte = byte;
                        TapeState::NextBit { mask: 0x80 }
                    } else {
                        TapeState::Pause
                    }
                }
                TapeState::NextBit { mask } => {
                    self.curr_bit = !self.curr_bit;
                    if (self.curr_byte & mask) == 0 {
                        self.delay = BIT_ZERO_LENGTH;
                        self.state = TapeState::BitHalf {
                            half_bit_delay: BIT_ZERO_LENGTH,
                            mask,
                        };
                    } else {
                        self.delay = BIT_ONE_LENGTH;
                        self.state = TapeState::BitHalf {
                            half_bit_delay: BIT_ONE_LENGTH,
                            mask,
                        };
                    };
                    break 'state_machine;
                }
                TapeState::BitHalf {
                    half_bit_delay,
                    mut mask,
                } => {
                    self.curr_bit = !self.curr_bit;
                    self.delay = half_bit_delay;
                    mask >>= 1;
                    self.state = if mask == 0 {
                        TapeState::NextByte
                    } else {
                        TapeState::NextBit { mask }
                    };
                    break 'state_machine;
                }
                TapeState::Pause => {
                    self.curr_bit = !self.curr_bit;
                    self.delay = PAUSE_LENGTH;
                    // Next block or end of the tape
                    self.state = TapeState::Play;
                    break 'state_machine;
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
        self.bufer_offset = 0;
        self.current_block_size = None;
        self.delay = 0;
        self.asset.seek(SeekFrom::Start(0))?;
        self.tape_ended = false;
        Ok(())
    }
}
