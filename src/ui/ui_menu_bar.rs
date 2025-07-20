use crate::{
    app::AppAction,
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
                            if ui.button("New Project").clicked() {
                                action.new_project();
                            }
                            ui.separator();
                            if ui.button("Open").clicked() {
                                action.open();
                            }
                            ui.separator();
                            if ui.add_enabled(enable_save, Button::new("Save")).clicked() {
                                action.save();
                            }
                            if ui.button("Save As").clicked() {
                                action.save_as();
                            }
                            ui.separator();
                            if ui.button("Quit").clicked() {
                                action.quit();
                            }
                        });
                    }

                    ui.menu_button("Edit", |ui| {
                        if ui
                            .add_enabled(action.has_undo(), Button::new("\u{27f2} Undo"))
                            .clicked()
                        {
                            action.undo();
                        }
                        if ui
                            .add_enabled(action.has_redo(), Button::new("\u{27f3} Redo"))
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
