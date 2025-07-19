use super::AudioSource;
use rubato::{FastFixedIn, Resampler as _};
use std::{
    collections::VecDeque,
    ops::AddAssign,
    sync::{Arc, Mutex},
};

pub struct ResampledLoopAudio {
    source: AudioSource,
    source_cursor: usize,
    resampler: FastFixedIn<f32>,
    resampler_input_buffer: Vec<Vec<f32>>,
    resampler_output_buffer: Vec<Vec<f32>>,
    output_sample_rate: u32,
    volume: Arc<Mutex<f32>>,
    pitch: Arc<Mutex<f32>>,
    output_buffer: VecDeque<f32>,
    last_volume: f32,
    last_pitch: f32,
}

impl ResampledLoopAudio {
    pub fn new(
        source: AudioSource,
        output_sample_rate: u32,
        chunk_size: usize,
        volume: Arc<Mutex<f32>>,
        pitch: Arc<Mutex<f32>>,
    ) -> anyhow::Result<Self> {
        println!(
            "source sample rate: {}, device sample rate: {}",
            source.sample_rate, output_sample_rate
        );
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
            output_sample_rate,
            volume,
            pitch,
            output_buffer: VecDeque::with_capacity(output_buffer_size),
            last_volume: 0.0,
            last_pitch: 0.0,
        })
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
            let pitch = self.pitch.lock().map(|v| *v).unwrap_or(self.last_pitch);
            self.last_pitch = pitch;
            let result = self
                .resampler
                .set_resample_ratio_relative(1.0 / pitch as f64, true);
            if result.is_err() {
                println!("{result:?}");
                break;
            }
            let result = self.resampler.process_into_buffer(
                &self.resampler_input_buffer,
                &mut self.resampler_output_buffer,
                None,
            );
            if result.is_err() {
                println!("{result:?}");
                break;
            }
            let output_len = result.unwrap_or_default().1;

            // 音量 (線形補間)
            let volume = self.volume.lock().map(|v| *v).unwrap_or(self.last_volume);
            self.last_volume = volume;

            // バッファに書き込み
            for i in 0..output_len {
                for channel in &self.resampler_output_buffer {
                    self.output_buffer.push_back(channel[i] * volume);
                }
            }
        }

        // バッファから出力へ移す
        for o in output.iter_mut() {
            *o += T::from_sample(self.output_buffer.pop_front().unwrap_or_default());
        }
    }
}
