use crate::{
    func_edit::FuncEdit as _,
    preference::Preference,
    state::{AudioEntry, SoundType, State, TrainPerformance},
};
use egui::{Context, Key, Modifiers};
use std::time::Instant;

pub struct PlayerState {
    pub master_controller: i32,
    pub speed: f64,
    pub sound_type: SoundType,
    smoothed_acceleration: f64,
    last_frame_time: Option<Instant>,
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
            sound_type: SoundType::Accel,
            smoothed_acceleration: 0.0,
            last_frame_time: None,
        }
    }
}

impl PlayerState {
    pub fn check(&mut self, train_performance: &TrainPerformance) {
        self.master_controller = self.master_controller.clamp(
            -(train_performance.brake_steps as i32),
            train_performance.power_steps as i32,
        );
        match self.master_controller.cmp(&0) {
            std::cmp::Ordering::Greater => self.sound_type = SoundType::Accel,
            std::cmp::Ordering::Less => self.sound_type = SoundType::Brake,
            std::cmp::Ordering::Equal => {}
        }
    }

    pub fn get_volume_pitch(&self, entry: &AudioEntry, preference: &Preference) -> (f32, f32) {
        if let Some(funcs) = entry.funcs_by_type(self.sound_type) {
            let volume = if self.master_controller != 0 && self.speed > 1e-6 {
                funcs.volume.value_at(self.speed)
            } else {
                0.0
            };
            let pitch = funcs.pitch.value_at(self.speed);
            (preference.global_volume * volume as f32, pitch as f32)
        } else {
            (0.0, 1.0)
        }
    }

    pub fn update(&mut self, ctx: &Context, state: &State, _preference: &Preference) {
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
            let mut acceleration = 0.0;

            match self.master_controller.cmp(&0) {
                std::cmp::Ordering::Greater => {
                    let f = self.master_controller as f64 / performance.power_steps as f64;
                    if let Some(a) = performance.acceleration.checked_value_at(self.speed) {
                        acceleration += f * a;
                    }
                }
                std::cmp::Ordering::Less => {
                    let f = -self.master_controller as f64 / performance.brake_steps as f64;
                    acceleration -= f * performance.brake_acceleration;
                }
                std::cmp::Ordering::Equal => {}
            }

            if let Some(drag) = performance.drag.checked_value_at(self.speed) {
                acceleration -= drag;
            }

            let sa = &mut self.smoothed_acceleration;
            *sa = acceleration.clamp(*sa - 4.0 * dt, *sa + 4.0 * dt);
            self.speed += *sa * dt;
        }
        self.speed = self.speed.max(0.0);

        self.last_frame_time = Some(std::time::Instant::now());
        ctx.request_repaint();
    }
}
