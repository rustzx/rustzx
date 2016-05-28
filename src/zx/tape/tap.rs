use super::*;
use std::fs::File;
use std::io::Read;
use utils::{make_word, Clocks};

const PILOT_LENGTH: usize = 2168;
const PILOT_PULSES_HEADER: usize = 8063;
const PILOT_PULSES_DATA: usize = 3223;
const SYNC1_LENGTH: usize = 667;
const SYNC2_LENGTH: usize = 735;
const BIT_ONE_LENGTH: usize = 1710;
const BIT_ZERO_LENGTH: usize = 855;
const PAUSE_LENGTH: usize = 3_500_000;

#[derive(PartialEq, Eq, Clone, Copy)]
enum TapeState {
    Stop,
    Play,
    Pilot,
    Sync,
    NextByte,
    NextBit,
    BitHalf(usize),
    Pause,
}

pub struct Tap {
    /// state of tape
    state: TapeState,
    /// previous state
    prev_state: TapeState,
    /// data of tape
    data: Vec<u8>,
    /// fields for pulse making from byte
    curr_bit: bool,
    curr_byte: u8,
    curr_mask: u8,
    // pulses left to next state
    pulse_counter: usize,
    /// pos in tape
    pos: usize,
    /// block info
    block: usize,
    block_size: usize,
    /// between-state timings
    delay: Clocks,
    acc_clocks: Clocks,
}

impl Tap {
    /// returns new *Tap* Tape instance
    pub fn new() -> Tap {
        Tap {
            prev_state: TapeState::Stop,
            state: TapeState::Stop,
            data: Vec::new(),
            curr_bit: true,
            curr_byte: 0x00,
            curr_mask: 0x80,
            pulse_counter: 0,
            pos: 0,
            block: 0,
            block_size: 0,
            delay: Clocks(0),
            acc_clocks: Clocks(0),
        }
    }
    fn reset_state(&mut self) {
        self.state = TapeState::Stop;
        self.curr_bit = true;
        self.curr_byte = 0x00;
        self.curr_mask = 0x80;
        self.pos = 0;
        self.block = 0;
        self.block_size = 0;
        self.delay = Clocks(0);
        self.acc_clocks = Clocks(0);
    }
    fn clear_data(&mut self) {
        self.data.clear();
    }
}

impl ZXTape for Tap {
    fn current_bit(&self) -> bool {
        self.curr_bit
    }
    fn insert(&mut self, path: &str) -> InsertResult
    {
        if let Ok(mut file) = File::open(path) {
            if let Err(_) = file.read_to_end(&mut self.data) {
                return InsertResult::Err("TAP file read error");
            }
            self.reset_state();
            return InsertResult::Ok;
        } else {
            return InsertResult::Err("Can't open TAP file");
        }

    }
    fn process_clocks(&mut self, clocks: Clocks) {
        let clocks = clocks.count();
        if self.state == TapeState::Stop {
            return;
        }
        // check delay
        if self.delay.count() > 0 {
            // accumulate clocks for delay
            self.acc_clocks += clocks;
            // if enough accumulated clocks then clear delay and drop some accumulated clocks
            if self.acc_clocks.count() >= self.delay.count() {
                self.acc_clocks -= self.delay;
                // self.acc_clocks = 0;
                self.delay = Clocks(0);
            }
            // return anyway, it is delay!
            return;
        } else {
            // clear accumulated clocks
            self.acc_clocks = Clocks(0);
        }
        // state machine. Wrapped into the loop for sequental non-clock-consuming state execution
        'state_machine: loop {
            match self.state {
                // Stop state.
                TapeState::Stop => {
                    // Tape stopped, return HI bit
                    self.curr_bit = true;
                    break 'state_machine;
                }
                // Play state. Starts the tape
                TapeState::Play => {
                    // out of range play
                    if self.pos >= self.data.len() {
                        // if play state happened when position is out of range,
                        // loop will be breaked on next iteration
                        self.state = TapeState::Stop;
                    } else {
                        // get block size as lsb word
                        self.block_size =
                            make_word(self.data[self.pos + 1], self.data[self.pos]) as usize;
                        // select appropriate pulse count for Pilot sequence
                        self.pulse_counter = if self.data[self.pos + 2] < 128 {
                            PILOT_PULSES_HEADER
                        } else {
                            PILOT_PULSES_DATA
                        };
                        // skip length bytes
                        self.pos += 2;
                        // so, ok seems to be ok, we can make output bit low
                        self.curr_bit = false;
                        // set delay before next state to one pilot pulse
                        self.delay = Clocks(PILOT_LENGTH);
                        self.state = TapeState::Pilot;
                        // break state machine, delay must be emulated
                        break 'state_machine;
                    }
                }
                // Pilot pulses
                TapeState::Pilot => {
                    // invert bit;
                    self.curr_bit = !self.curr_bit;
                    // one pulse passed
                    self.pulse_counter -= 1;
                    if self.pulse_counter > 0 {
                        // add new delay and break
                        self.delay = Clocks(PILOT_LENGTH);
                    } else {
                        // change state to first sync
                        self.state = TapeState::Sync;
                        self.delay = Clocks(SYNC1_LENGTH);
                    }
                    // break anyway for delay
                    break 'state_machine;
                }
                // sync pulse
                TapeState::Sync => {
                    self.curr_bit = !self.curr_bit;
                    self.delay = Clocks(SYNC2_LENGTH);
                    self.state = TapeState::NextByte;
                    break 'state_machine;
                }
                // read next byte
                TapeState::NextByte => {
                    // read from most singificant bit
                    self.curr_mask = 0x80;
                    self.curr_byte = 0x00;
                    // break not needed, state doesn't require any time
                    self.state = TapeState::NextBit;
                }
                // next bit
                TapeState::NextBit => {
                    // invert bit
                    self.curr_bit = !self.curr_bit;
                    // depending on bit state select timing and switch to new state
                    if (self.data[self.pos] & self.curr_mask) == 0 {
                        self.delay = Clocks(BIT_ZERO_LENGTH);
                        self.state = TapeState::BitHalf(BIT_ZERO_LENGTH);
                    } else {
                        self.delay = Clocks(BIT_ONE_LENGTH);
                        self.state = TapeState::BitHalf(BIT_ONE_LENGTH);
                        self.curr_byte |= self.curr_mask & 0xFF;
                    };
                    break 'state_machine;
                }
                // second half of bit
                TapeState::BitHalf(pulse_length) => {
                    self.curr_bit = !self.curr_bit;
                    // set timeout same as before
                    self.delay = Clocks(pulse_length);
                    // shift right, to the next bit
                    self.curr_mask >>= 1;
                    if self.curr_mask == 0 {
                        self.pos += 1;
                        self.block_size -= 1;
                        self.state = if self.block_size > 0 {
                            TapeState::NextByte
                        } else {
                            TapeState::Pause
                        };
                    } else {
                        self.state = TapeState::NextBit;
                    }
                    break 'state_machine;
                }
                // pause after block
                TapeState::Pause => {
                    self.curr_bit = !self.curr_bit;
                    // if we still have blocks
                    if self.pos < self.data.len() {
                        // next block
                        self.delay = Clocks(PAUSE_LENGTH);
                        self.block += 1;
                        self.state = TapeState::Play;
                        // break directly for delay
                        break 'state_machine;
                    } else {
                        // stop the tape. loop will be breaked on next state
                        self.state = TapeState::Stop;
                    }
                }
            }
        }
    }
    fn eject(&mut self) {
        self.clear_data();
        self.reset_state();
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
                let prev_state = self.prev_state;
                self.state = prev_state;
            }
        }
    }
    fn rewind(&mut self) {}
}
