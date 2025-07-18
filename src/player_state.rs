use crate::state::TrainPerformance;
use egui::{Context, Key, Modifiers};

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub master_controller: i32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            master_controller: i32::MIN,
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
        // マスコン
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
    }
}
