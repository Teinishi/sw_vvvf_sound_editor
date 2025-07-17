use crate::{state::State, ui::ui_about_rev};
use egui::{Layout, Window};

#[derive(Debug, Default)]
pub struct UiSettingWindow;

impl UiSettingWindow {
    pub const TITLE: &str = "Settings";

    pub fn show(&self, ctx: &egui::Context, open: &mut bool, state: &mut State) {
        Window::new(Self::TITLE)
            .open(open)
            .default_size([300.0, 200.0])
            .min_size([200.0, 100.0])
            .show(ctx, |ui| {
                self.ui(ui, state);
            });
    }

    #[expect(clippy::unused_self)]
    #[expect(unused_variables)]
    #[expect(clippy::needless_pass_by_ref_mut)]
    fn ui(&self, ui: &mut egui::Ui, state: &mut State) {
        egui::widgets::global_theme_preference_buttons(ui);

        ui.with_layout(Layout::bottom_up(egui::Align::Min), |ui| {
            ui_about_rev(ui);
            ui.separator();
        });
    }
}
