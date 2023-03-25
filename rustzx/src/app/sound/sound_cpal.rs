//! Real Audio SDL backend
use crate::app::{
    settings::Settings,
    sound::{SoundDevice, ZXSample, CHANNEL_COUNT, ringbuf_size_from_sample_rate},
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;

pub struct SoundCpal {
    tx: ringbuf::Producer<ZXSample, Arc<ringbuf::HeapRb::<ZXSample>>>,
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

                // Find any stereo config
                (c.channels() as usize % CHANNEL_COUNT == 0) && c.channels() != 0
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

        let ringbuf_size = ringbuf_size_from_sample_rate(sample_rate);
        let ringbuf = ringbuf::HeapRb::<ZXSample>::new(ringbuf_size);
        let (tx, rx) = ringbuf.split();

        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => create_stream::<i16>(&device, &config.into(), rx)?,
            cpal::SampleFormat::U16 => create_stream::<u16>(&device, &config.into(), rx)?,
            cpal::SampleFormat::F32 => create_stream::<f32>(&device, &config.into(), rx)?,
            cpal::SampleFormat::I8 => create_stream::<i8>(&device, &config.into(), rx)?,
            cpal::SampleFormat::I32 => create_stream::<i32>(&device, &config.into(), rx)?,
            cpal::SampleFormat::I64 => create_stream::<i64>(&device, &config.into(), rx)?,
            cpal::SampleFormat::U8 => create_stream::<u8>(&device, &config.into(), rx)?,
            cpal::SampleFormat::U32 => create_stream::<u32>(&device, &config.into(), rx)?,
            cpal::SampleFormat::U64 => create_stream::<u64>(&device, &config.into(), rx)?,
            cpal::SampleFormat::F64 => create_stream::<f64>(&device, &config.into(), rx)?,
            _ => {
                anyhow::bail!("Device has unsupported audio sample format")
            },

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
        // Ignore buffer overflows
        let _ = self.tx.push(sample);
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}

fn create_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut samples_rx: ringbuf::Consumer<ZXSample, Arc<ringbuf::HeapRb::<ZXSample>>>,
) -> anyhow::Result<cpal::Stream>
where
    T: cpal::FromSample<f32> + cpal::SizedSample,
{
    let channels = config.channels as usize;

    let stream = device.build_output_stream(
        config,
        move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in out.chunks_mut(channels) {
                match samples_rx.pop() {
                    Some(zx_sample) => {
                        let left: T = cpal::Sample::from_sample(zx_sample.left);
                        let right: T = cpal::Sample::from_sample(zx_sample.right);
                        frame[0] = left;
                        frame[1] = right;
                    }
                    None => {
                        frame[0] = cpal::Sample::EQUILIBRIUM;
                        frame[1] = cpal::Sample::EQUILIBRIUM;
                    }
                }

                // We use only first stereo channels, other channels should be silent
                frame[2..channels].fill(cpal::Sample::EQUILIBRIUM);
            }
        },
        |_| {},
        None,
    )?;
    stream.play()?;
    Ok(stream)
}
