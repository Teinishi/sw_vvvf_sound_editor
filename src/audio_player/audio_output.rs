use anyhow::bail;
use cpal::traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _};
use std::ops::AddAssign;

pub struct AudioOutput {
    device: cpal::Device,
    stream: Option<cpal::Stream>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Failed to find output device.");

        Self {
            device,
            stream: None,
        }
    }
}

impl AudioOutput {
    pub fn play<F>(&mut self, data_callback: F) -> anyhow::Result<()>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let config = self.device.default_output_config()?;
        match config.sample_format() {
            cpal::SampleFormat::I8 => self.run::<i8, _>(&config.into(), data_callback),
            cpal::SampleFormat::I16 => self.run::<i16, _>(&config.into(), data_callback),
            //cpal::SampleFormat::I24 => self.run::<cpal::I24, _>(&config.into(), data_callback),
            cpal::SampleFormat::I32 => self.run::<i32, _>(&config.into(), data_callback),
            // cpal::SampleFormat::I48 => self.run::<I48, _>(&config.into(), data_callback),
            cpal::SampleFormat::I64 => self.run::<i64, _>(&config.into(), data_callback),
            cpal::SampleFormat::U8 => self.run::<u8, _>(&config.into(), data_callback),
            cpal::SampleFormat::U16 => self.run::<u16, _>(&config.into(), data_callback),
            // cpal::SampleFormat::U24 => self.run::<U24, _>(&config.into(), data_callback),
            cpal::SampleFormat::U32 => self.run::<u32, _>(&config.into(), data_callback),
            // cpal::SampleFormat::U48 => self.run::<U48, _>(&config.into(), data_callback),
            cpal::SampleFormat::U64 => self.run::<u64, _>(&config.into(), data_callback),
            cpal::SampleFormat::F32 => self.run::<f32, _>(&config.into(), data_callback),
            cpal::SampleFormat::F64 => self.run::<f64, _>(&config.into(), data_callback),
            sample_format => bail!("Unsupported sample format '{sample_format}'"),
        }
    }

    fn run<T, F>(&mut self, config: &cpal::StreamConfig, mut data_callback: F) -> anyhow::Result<()>
    where
        T: cpal::SizedSample + cpal::FromSample<f32> + AddAssign,
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let stream = self.device.build_output_stream(
            config,
            move |output: &mut [T], _| {
                let mut buffer = vec![0.0f32; output.len()];
                data_callback(&mut buffer);
                for (out, &sample) in output.iter_mut().zip(buffer.iter()) {
                    *out = T::from_sample(sample);
                }
            },
            |err| eprintln!("an error occurred on stream: {err}"),
            None,
        )?;
        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }
}
