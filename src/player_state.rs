use crate::{audio_player::AudioOutput, state::TrainPerformance};
use egui::{Context, Key, Modifiers};
use std::time::Instant;

pub struct PlayerState {
    pub master_controller: i32,
    pub speed: f64,
    last_frame_time: Option<Instant>,
    audio_output: AudioOutput,
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
        }
    }
}

impl PlayerState {
    pub fn play(&mut self) -> anyhow::Result<()> {
        let mut phase: f32 = 0.0;
        let step = 440.0 / 48000.0 * 2.0 * std::f32::consts::PI;

        self.audio_output.play(move |data| {
            for frame in data.chunks_mut(2) {
                let v = phase.sin() * 0.2;
                phase += step;

                for o in frame {
                    *o = v;
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

    pub fn update(&mut self, ctx: &Context, train_performance: &TrainPerformance) {
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

        self.check(train_performance);

        // 速度更新
        if let Some(dt) = self.last_frame_time.map(|i| i.elapsed().as_secs_f64()) {
            match self.master_controller.cmp(&0) {
                std::cmp::Ordering::Greater => {
                    let f = self.master_controller as f64 / train_performance.power_steps as f64;
                    if let Some(a) = train_performance.acceleration.checked_value_at(self.speed) {
                        self.speed += f * a * dt;
                    }
                }
                std::cmp::Ordering::Less => {
                    let f = -self.master_controller as f64 / train_performance.brake_steps as f64;
                    self.speed -= f * train_performance.brake_acceleration * dt;
                }
                std::cmp::Ordering::Equal => {}
            }

            if let Some(drag) = train_performance.drag.checked_value_at(self.speed) {
                self.speed -= drag * dt;
            }
        }
        self.speed = self.speed.max(0.0);

        // 各音声の音量とピッチを更新

        self.last_frame_time = Some(std::time::Instant::now());
        ctx.request_repaint();
    }
}
