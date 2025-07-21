use crate::func_edit::{EditableFunc, EditablePositiveFunc, FuncEdit as _};

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct TrainPerformance {
    pub acceleration: EditablePositiveFunc,
    pub power_steps: u8,
    pub brake_acceleration: f64,
    pub brake_steps: u8,
    pub drag: EditablePositiveFunc,
}

impl Default for TrainPerformance {
    fn default() -> Self {
        Self {
            acceleration: EditableFunc::with_expression("min(2.5,90/x,(80/x)^2)").into(),
            power_steps: 5,
            brake_acceleration: 4.2,
            brake_steps: 8,
            drag: EditableFunc::with_expression("x/500").into(),
        }
    }
}

impl TrainPerformance {
    pub fn update(&mut self) {
        self.acceleration.update_expression();
        self.drag.update_expression();
    }
}
