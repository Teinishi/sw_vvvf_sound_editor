use super::AudioSource;
use rubato::{FastFixedIn, Resampler as _};
use std::{collections::VecDeque, ops::AddAssign};

pub struct ResampledLoopAudio {
    source: AudioSource,
    source_cursor: usize,
    resampler: FastFixedIn<f32>,
    resampler_input_buffer: Vec<Vec<f32>>,
    resampler_output_buffer: Vec<Vec<f32>>,
    volume: f32,
    pitch: f32,
    output_buffer: VecDeque<f32>,
}

impl ResampledLoopAudio {
    pub fn new(
        source: AudioSource,
        output_sample_rate: u32,
        chunk_size: usize,
    ) -> anyhow::Result<Self> {
        let channels = source.channels;
        let resampler = FastFixedIn::new(
            output_sample_rate as f64 / source.sample_rate as f64,
            10.0,
            rubato::PolynomialDegree::Cubic,
            chunk_size,
            channels,
        )?;
        let resampler_input_buffer = resampler.input_buffer_allocate(true);
        let resampler_output_buffer = resampler.output_buffer_allocate(true);
        let output_buffer_size = resampler_output_buffer.iter().map(|b| b.len()).sum();
        Ok(Self {
            source,
            source_cursor: 0,
            resampler,
            resampler_input_buffer,
            resampler_output_buffer,
            volume: 1.0,
            pitch: 1.0,
            output_buffer: VecDeque::with_capacity(output_buffer_size),
        })
    }

    pub fn set_volume_pitch(&mut self, volume: f32, pitch: f32) {
        self.volume = volume;
        self.pitch = pitch;
    }

    pub fn write_data_additive<T>(&mut self, output: &mut [T])
    where
        T: cpal::Sample + cpal::FromSample<f32> + AddAssign,
    {
        // バッファが足りなければ取得
        while self.output_buffer.len() < output.len() {
            // 先のフレームをsourceから取得 端までいったらループ
            let c = self.source_cursor;
            let l = self.source.len;
            for (buffer, source) in self
                .resampler_input_buffer
                .iter_mut()
                .zip(self.source.samples.iter())
            {
                for (i, b) in buffer.iter_mut().enumerate() {
                    *b = source[(c + i) % l];
                }
            }
            self.source_cursor += self.resampler_input_buffer[0].len();
            self.source_cursor %= l;

            // リサンプル
            if let Ok(result) = self
                .resampler
                .set_resample_ratio_relative(1.0 / self.pitch as f64, true)
                .and_then(|_| {
                    self.resampler.process_into_buffer(
                        &self.resampler_input_buffer,
                        &mut self.resampler_output_buffer,
                        None,
                    )
                })
            {
                let output_len = result.1;
                // バッファに書き込み
                for i in 0..output_len {
                    for channel in &self.resampler_output_buffer {
                        self.output_buffer.push_back(channel[i] * self.volume);
                    }
                }
            } else {
                break;
            }
        }

        // バッファから出力へ移す
        for o in output.iter_mut() {
            *o += T::from_sample(self.output_buffer.pop_front().unwrap_or_default());
        }
    }
}
