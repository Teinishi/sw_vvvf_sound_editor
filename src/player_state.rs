use crate::state::TrainPerformance;
use egui::{Context, Key, Modifiers};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub master_controller: i32,
    pub speed: f64,
    last_frame_time: Option<Instant>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            master_controller: i32::MIN,
            speed: 0.0,
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

        self.last_frame_time = Some(std::time::Instant::now());
        ctx.request_repaint();
    }
}
