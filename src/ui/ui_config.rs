use crate::state::State;

#[derive(Debug, Default)]
pub struct UiConfig;

impl UiConfig {
    pub fn ui(&self, ui: &mut egui::Ui, state: &mut State) {
        ui.strong("Config");
    }
}
