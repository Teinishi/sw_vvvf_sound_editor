use crate::{
    app::AppAction,
    ui::{UiAudioFiles, UiPerformanceWindow, UiPitchVolumeEdit, UiSettingWindow},
};
use egui::{Button, MenuBar, Sides};

#[derive(Debug, Default)]
pub struct UiMenuBar;

impl UiMenuBar {
    #[expect(clippy::unused_self)]
    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        enable_save: bool,
        show_audio_files_panel: &mut bool,
        show_point_edit_panel: &mut bool,
        show_performance_window: &mut bool,
        show_setting_window: &mut bool,
    ) {
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

                    ui.toggle_value(show_audio_files_panel, UiAudioFiles::TITLE);
                    ui.toggle_value(show_point_edit_panel, UiPitchVolumeEdit::TITLE);
                    ui.toggle_value(show_performance_window, UiPerformanceWindow::TITLE);
                },
                |ui| {
                    ui.toggle_value(show_setting_window, UiSettingWindow::TITLE);
                },
            )
        });
    }
}
