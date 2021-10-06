//! Real Audio SDL backend
use crate::app::{
    settings::Settings,
    sound::{SoundDevice, ZXSample, CHANNEL_COUNT},
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

pub struct SoundCpal {
    tx: mpsc::Sender<ZXSample>,
    sample_rate: usize,
    // Keep stream alive until Drop
    _stream: cpal::Stream,
}

impl SoundCpal {
    /// Constructs sound backend from settings
    pub fn new(settings: &Settings) -> anyhow::Result<SoundCpal> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("Failed to acquire cpal sound host"))?;

        let config = device
            .supported_output_configs()?
            .find(|c| {
                if let Some(sample_rate) = settings.sound_sample_rate {
                    if sample_rate < c.min_sample_rate().0 as usize
                        || sample_rate > c.max_sample_rate().0 as usize
                    {
                        return false;
                    }
                }

                c.channels() == CHANNEL_COUNT as u16
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Sound device does not support required configuration")
            })?;

        let config = if let Some(sample_rate) = settings.sound_sample_rate {
            config.with_sample_rate(cpal::SampleRate(sample_rate as u32))
        } else {
            config.with_max_sample_rate()
        };

        let sample_rate = config.sample_rate().0 as usize;

        let (tx, rx) = mpsc::channel();

        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => create_stream::<i16>(&device, &config.into(), rx)?,
            cpal::SampleFormat::U16 => create_stream::<u16>(&device, &config.into(), rx)?,
            cpal::SampleFormat::F32 => create_stream::<f32>(&device, &config.into(), rx)?,
        };

        Ok(SoundCpal {
            tx,
            sample_rate,
            _stream: stream,
        })
    }
}

impl SoundDevice for SoundCpal {
    fn send_sample(&mut self, sample: ZXSample) {
        self.tx.send(sample).unwrap();
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}

fn create_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    samples_rx: mpsc::Receiver<ZXSample>,
) -> anyhow::Result<cpal::Stream>
where
    T: cpal::Sample,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in out.chunks_mut(channels) {
                match samples_rx.try_recv().ok() {
                    Some(zx_sample) => {
                        let left: T = cpal::Sample::from(&zx_sample.left);
                        let right: T = cpal::Sample::from(&zx_sample.right);
                        frame[0] = left;
                        frame[1] = right;
                    }
                    None => {
                        frame[0] = cpal::Sample::from(&0f32);
                        frame[1] = cpal::Sample::from(&0f32);
                    }
                }
            }
        },
        |_| {},
    )?;
    stream.play()?;
    Ok(stream)
}
