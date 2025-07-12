#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct UiVolumePitchGraph {}

impl UiVolumePitchGraph {
    pub fn ui(&self, ui: &mut egui::Ui) {}
}
