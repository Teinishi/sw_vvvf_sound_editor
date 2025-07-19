use super::AudioMixer;
use cpal::traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _};
use std::{
    ops::AddAssign,
    sync::{Arc, Mutex},
};

pub struct AudioOutput {
    device: cpal::Device,
}

impl std::fmt::Debug for AudioOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioOutput")
    }
}

impl Default for AudioOutput {
    fn default() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Failed to find output device.");

        Self { device }
    }
}

impl AudioOutput {
    pub fn play(&self, stream: Arc<Mutex<AudioMixer>>) -> anyhow::Result<()> {
        let config = self
            .device
            .default_output_config()
            .expect("Failed to get output config.");
        match config.sample_format() {
            cpal::SampleFormat::I8 => self.run::<i8>(&config.into(), stream),
            cpal::SampleFormat::I16 => self.run::<i16>(&config.into(), stream),
            //cpal::SampleFormat::I24 => self.run::<cpal::I24>(&config.into(), stream),
            cpal::SampleFormat::I32 => self.run::<i32>(&config.into(), stream),
            // cpal::SampleFormat::I48 => self.run::<I48>(&config.into(), stream),
            cpal::SampleFormat::I64 => self.run::<i64>(&config.into(), stream),
            cpal::SampleFormat::U8 => self.run::<u8>(&config.into(), stream),
            cpal::SampleFormat::U16 => self.run::<u16>(&config.into(), stream),
            // cpal::SampleFormat::U24 => self.run::<U24>(&config.into(), stream),
            cpal::SampleFormat::U32 => self.run::<u32>(&config.into(), stream),
            // cpal::SampleFormat::U48 => self.run::<U48>(&config.into(), stream),
            cpal::SampleFormat::U64 => self.run::<u64>(&config.into(), stream),
            cpal::SampleFormat::F32 => self.run::<f32>(&config.into(), stream),
            cpal::SampleFormat::F64 => self.run::<f64>(&config.into(), stream),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        }
    }

    fn run<T>(
        &self,
        config: &cpal::StreamConfig,
        stream: Arc<Mutex<AudioMixer>>,
    ) -> anyhow::Result<()>
    where
        T: cpal::SizedSample + cpal::FromSample<f32> + AddAssign,
    {
        /*let device_channels = config.channels as usize;

        let mut stream = OggStreamReader::new(File::open("run4.ogg").expect("Failed to open file"))
            .expect("Failed to decode");
        let source = AudioSource::from_ogg(&mut stream, device_channels)?;
        let mut stream = LoopAudioStream::new(
            source,
            config.sample_rate.0,
            256,
            |t| 0.1 + 0.1 * t,
            |t| 0.1 + 0.1 * t,
        )?;*/

        let stream = self.device.build_output_stream(
            config,
            move |data: &mut [T], _| {
                if let Ok(mut stream) = stream.lock() {
                    stream.write_data(data);
                }
            },
            |err| eprintln!("an error occurred on stream: {err}"),
            None,
        )?;
        stream.play()?;

        Ok(())
    }
}
