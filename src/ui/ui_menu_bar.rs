use crate::{
    app_action::AppAction,
    ui::{UiAudioFiles, UiPerformanceWindow, UiPitchVolumeEdit, UiSettingWindow},
};
use egui::{Button, MenuBar, Sides};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UiMenuBar {
    pub show_audio_files_panel: bool,
    pub show_point_edit_panel: bool,
    pub show_performance_window: bool,
    pub show_setting_window: bool,
}

impl Default for UiMenuBar {
    fn default() -> Self {
        Self {
            show_audio_files_panel: true,
            show_point_edit_panel: true,
            show_performance_window: false,
            show_setting_window: false,
        }
    }
}

impl UiMenuBar {
    pub fn ui(&mut self, ui: &mut egui::Ui, action: &mut AppAction, enable_save: bool) {
        MenuBar::new().ui(ui, |ui| {
            Sides::new().show(
                ui,
                |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.menu_button("File", |ui| {
                            if button_with_shortcut(ui, "New Project", "Ctrl+N", true).clicked() {
                                action.new_project();
                            }
                            ui.separator();
                            if button_with_shortcut(ui, "Open", "Ctrl+O", true).clicked() {
                                action.open();
                            }
                            ui.separator();
                            if button_with_shortcut(ui, "Save", "Ctrl+S", enable_save).clicked() {
                                action.save();
                            }
                            if button_with_shortcut(ui, "Save As", "Ctrl+Shift+N", true).clicked() {
                                action.save_as();
                            }
                            ui.separator();
                            if ui.button("Quit").clicked() {
                                action.quit();
                            }
                        });
                    }

                    ui.menu_button("Edit", |ui| {
                        if button_with_shortcut(ui, "\u{27f2} Undo", "Ctrl+Z", action.has_undo())
                            .clicked()
                        {
                            action.undo();
                        }
                        if button_with_shortcut(ui, "\u{27f3} Redo", "Ctrl+Y", action.has_redo())
                            .clicked()
                        {
                            action.redo();
                        }
                    });

                    ui.separator();

                    ui.toggle_value(&mut self.show_audio_files_panel, UiAudioFiles::TITLE);
                    ui.toggle_value(&mut self.show_point_edit_panel, UiPitchVolumeEdit::TITLE);
                    ui.toggle_value(
                        &mut self.show_performance_window,
                        UiPerformanceWindow::TITLE,
                    );
                },
                |ui| {
                    ui.toggle_value(&mut self.show_setting_window, UiSettingWindow::TITLE);
                },
            )
        });
    }
}

fn button_with_shortcut(
    ui: &mut egui::Ui,
    text: &str,
    shortcut: &str,
    enabled: bool,
) -> egui::Response {
    ui.add_enabled(
        enabled,
        Button::new(text).right_text(egui::RichText::new(shortcut).weak()),
    )
}
