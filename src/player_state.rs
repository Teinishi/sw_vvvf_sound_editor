use crate::{
    audio_player::{AudioOutput, AudioSource, ResampledLoopAudio},
    state::{AudioEntryId, State, TrainPerformance},
};
use egui::{Context, Key, Modifiers};
use lewton::inside_ogg::OggStreamReader;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

fn new_audio_source(
    path: &PathBuf,
    config: &cpal::StreamConfig,
) -> anyhow::Result<ResampledLoopAudio> {
    let file = File::open(path)?;
    let mut stream = OggStreamReader::new(file)?;
    let source = AudioSource::from_ogg(&mut stream, config.channels as usize)?;
    ResampledLoopAudio::new(source, config.sample_rate.0, 256)
}

pub struct PlayerState {
    pub master_controller: i32,
    pub speed: f64,
    last_frame_time: Option<Instant>,
    audio_output: AudioOutput,
    audio_sources: Arc<Mutex<HashMap<AudioEntryId, anyhow::Result<ResampledLoopAudio>>>>,
}

impl std::fmt::Debug for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[derive(Debug)]
        struct PlayerState {
            #[expect(dead_code)]
            master_controller: i32,
            #[expect(dead_code)]
            speed: f64,
        }
        let s = PlayerState {
            master_controller: self.master_controller,
            speed: self.speed,
        };
        write!(f, "{s:?}")
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            master_controller: i32::MIN,
            speed: 0.0,
            last_frame_time: None,
            audio_output: AudioOutput::default(),
            audio_sources: Default::default(),
        }
    }
}

impl PlayerState {
    pub fn play(&mut self) -> anyhow::Result<()> {
        let sources = self.audio_sources.clone();

        self.audio_output.play(move |data| {
            if let Ok(mut sources) = sources.lock() {
                for entry in sources.values_mut().flatten() {
                    entry.write_data_additive(data);
                }
            }
        })
    }

    pub fn check(&mut self, train_performance: &TrainPerformance) {
        self.master_controller = self.master_controller.clamp(
            -(train_performance.brake_steps as i32),
            train_performance.power_steps as i32,
        );
    }

    pub fn update(&mut self, ctx: &Context, state: &State) {
        let performance = &state.train_performance;

        // マスコン キー入力
        let (key_1, key_q, key_a, key_z, key_s) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::Num1),
                i.consume_key(Modifiers::NONE, Key::Q),
                i.consume_key(Modifiers::NONE, Key::A),
                i.consume_key(Modifiers::NONE, Key::Z),
                i.consume_key(Modifiers::NONE, Key::S),
            )
        });
        let m = &mut self.master_controller;
        if key_1 {
            *m = i32::MIN;
        }
        if key_q {
            *m = m.saturating_sub(1);
        }
        if key_a {
            *m = m.saturating_add(-m.signum());
        }
        if key_z {
            *m = m.saturating_add(1);
        }
        if key_s {
            *m = 0;
        }

        self.check(performance);

        // 速度更新
        if let Some(dt) = self.last_frame_time.map(|i| i.elapsed().as_secs_f64()) {
            match self.master_controller.cmp(&0) {
                std::cmp::Ordering::Greater => {
                    let f = self.master_controller as f64 / performance.power_steps as f64;
                    if let Some(a) = performance.acceleration.checked_value_at(self.speed) {
                        self.speed += f * a * dt;
                    }
                }
                std::cmp::Ordering::Less => {
                    let f = -self.master_controller as f64 / performance.brake_steps as f64;
                    self.speed -= f * performance.brake_acceleration * dt;
                }
                std::cmp::Ordering::Equal => {}
            }

            if let Some(drag) = performance.drag.checked_value_at(self.speed) {
                self.speed -= drag * dt;
            }
        }
        self.speed = self.speed.max(0.0);

        // 音声データの過不足チェック、音量ピッチ更新
        if let Ok(mut sources) = self.audio_sources.lock() {
            let mut to_remove: HashSet<AudioEntryId> = sources.keys().cloned().collect();

            if let Ok(config) = self.audio_output.config() {
                let config = &config.into();
                for entry in &state.audio_entries {
                    let path = entry.path();
                    to_remove.remove(path);

                    if !sources.contains_key(path) {
                        sources.insert(path.clone(), new_audio_source(path, config));
                    }
                    if let Some(Ok(source)) = sources.get_mut(path) {
                        // 各音声の音量とピッチを更新
                        let volume = if self.master_controller != 0 && self.speed > 1e-6 {
                            entry.volume().value_at(self.speed)
                        } else {
                            0.0
                        };
                        let pitch = entry.pitch().value_at(self.speed);
                        source.set_volume_pitch(volume as f32, pitch as f32);
                    }
                }
            }

            sources.retain(|k, _| !to_remove.contains(k));
        }

        self.last_frame_time = Some(std::time::Instant::now());
        ctx.request_repaint();
    }
}
