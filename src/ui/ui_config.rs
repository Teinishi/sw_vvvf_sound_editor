use crate::{state::State, ui::ui_about_rev};
use egui::Layout;

#[derive(Debug, Default)]
pub struct UiConfig;

impl UiConfig {
    #[expect(clippy::unused_self)]
    pub fn ui(&self, ui: &mut egui::Ui, state: &mut State) {
        ui.strong("Config");

        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            ui_about_rev(ui);
        });
    }
}
