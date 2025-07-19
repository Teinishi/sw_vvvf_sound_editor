use crate::audio_player::ResampledLoopAudio;
use std::ops::AddAssign;

#[derive(Default)]
pub struct AudioMixer {
    sources: Vec<ResampledLoopAudio>,
}

impl std::fmt::Debug for AudioMixer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AudioMixer")
    }
}

impl AudioMixer {
    pub fn write_data<T>(&mut self, output: &mut [T])
    where
        T: cpal::Sample + cpal::FromSample<f32> + AddAssign,
    {
        for source in &mut self.sources {
            source.write_data_additive(output);
        }
    }
}
