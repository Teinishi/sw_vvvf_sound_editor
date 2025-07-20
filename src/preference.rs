#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Preference {
    pub global_volume: f32,
}

impl Default for Preference {
    fn default() -> Self {
        Self { global_volume: 1.0 }
    }
}
