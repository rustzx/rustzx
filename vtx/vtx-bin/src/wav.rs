use std::{fs::File, path::PathBuf, usize};
use structopt::StructOpt;
use vtx::{player::PrecisePlayer, Vtx};

#[derive(StructOpt)]
pub struct ConvertWav {
    /// `*.vtx` input file path
    #[structopt(short, long)]
    input: PathBuf,
    /// `*.wav` output file path
    #[structopt(short, long)]
    output: PathBuf,
}

impl ConvertWav {
    pub fn execute(self) -> Result<(), anyhow::Error> {
        const CHUNK_SIZE: usize = 16 * 64000;
        const SAMPLE_RATE: usize = 44100;
        const CHANNEL_COUNT: usize = 2;
        const BITS_PER_SAMPLE: usize = 16;

        println!("Parsing vtx file...");
        let input_file = File::open(self.input)?;
        let vtx = Vtx::load(input_file)?;

        println!("Title: {}", vtx.title);
        println!("Author: {}", vtx.author);
        println!("From: {}", vtx.from);
        println!("Year: {}", vtx.year);
        println!("Tracker: {}", vtx.tracker);
        println!("Comment: {}", vtx.comment);

        let mut player = PrecisePlayer::new(vtx, SAMPLE_RATE, true);

        let mut buffer = vec![0i16; CHUNK_SIZE];
        let mut pos = 0;
        loop {
            let samples_generated = player.play(&mut buffer[pos..pos + CHUNK_SIZE]);
            if samples_generated == 0 {
                break;
            }
            pos += samples_generated;
            buffer.resize(pos + CHUNK_SIZE, 0);
        }

        buffer.truncate(pos);

        let wav_header = wav::Header::new(
            wav::WAV_FORMAT_PCM,
            CHANNEL_COUNT as u16,
            SAMPLE_RATE as u32,
            BITS_PER_SAMPLE as u16,
        );

        let mut output_file = File::create(self.output)?;
        wav::write(
            wav_header,
            &wav::BitDepth::Sixteen(buffer),
            &mut output_file,
        )?;
        println!("File successfully converted!");
        Ok(())
    }
}
