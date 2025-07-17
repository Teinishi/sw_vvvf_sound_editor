use crate::app::AppAction;
use egui::{Button, MenuBar, Sides};

#[derive(Debug, Default)]
pub struct UiMenuBar;

impl UiMenuBar {
    #[expect(clippy::unused_self)]
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        show_audio_files_panel: &mut bool,
        show_point_edit_panel: &mut bool,
        show_config_panel: &mut bool,
    ) {
        MenuBar::new().ui(ui, |ui| {
            Sides::new().show(
                ui,
                |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.menu_button("File", |ui| {
                            if ui.button("Quit").clicked() {
                                action.close();
                            }
                        });
                    }

                    ui.menu_button("Edit", |ui| {
                        if ui
                            .add_enabled(action.has_undo(), Button::new("\u{27f2}Undo"))
                            .clicked()
                        {
                            action.undo();
                        }
                        if ui
                            .add_enabled(action.has_redo(), Button::new("\u{27f3}Redo"))
                            .clicked()
                        {
                            action.redo();
                        }
                    });

                    ui.separator();

                    ui.toggle_value(show_audio_files_panel, "Audio Files");
                    ui.toggle_value(show_point_edit_panel, "Point Edit");
                },
                |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);

                    ui.separator();

                    ui.toggle_value(show_config_panel, "Config");
                },
            )
        });
    }
}
