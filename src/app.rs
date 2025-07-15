use crate::{
    state::State,
    ui::{UiAudioFiles, UiPitchVolumePlots, UiPointEdit},
};
use egui::{CentralPanel, Frame, MenuBar, SidePanel, Sides, TopBottomPanel, vec2};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MainApp {
    state: State,
    show_audio_files_panel: bool,
    show_point_edit_panel: bool,
    #[serde(skip)]
    ui_audio_files: UiAudioFiles,
    #[serde(skip)]
    ui_point_edit: UiPointEdit,
    #[serde(skip)]
    ui_pitch_volume_plots: UiPitchVolumePlots,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            state: State::default(),
            show_audio_files_panel: true,
            show_point_edit_panel: true,
            ui_audio_files: UiAudioFiles,
            ui_point_edit: UiPointEdit,
            ui_pitch_volume_plots: UiPitchVolumePlots::default(),
        }
    }
}

impl MainApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        #[expect(clippy::large_include_file)]
        fonts.font_data.insert(
            "noto_sans_jp_regular".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../fonts/NotoSansJP-Regular.ttf"
            ))),
        );
        fonts.font_data.insert(
            "roboto_regular".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../fonts/Roboto-Regular.ttf"
            ))),
        );
        let font_families = fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .expect("Failed to init fonts.");
        font_families.insert(0, "roboto_regular".to_owned());
        font_families.insert(1, "noto_sans_jp_regular".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for MainApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                Sides::new().show(
                    ui,
                    |ui| {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            ui.menu_button("File", |ui| {
                                if ui.button("Quit").clicked() {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            });
                        }

                        ui.separator();

                        ui.toggle_value(&mut self.show_audio_files_panel, "Audio Files");
                        ui.toggle_value(&mut self.show_point_edit_panel, "Point Edit");
                    },
                    |ui| {
                        egui::widgets::global_theme_preference_buttons(ui);
                    },
                )
            });
        });

        /*if let Some(path) = &self.work_folder {
            if let Some(path_str) = path.to_str() {
                egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.weak(path_str);
                    ui.add_space(2.0);
                });
            }
        }*/

        if self.show_audio_files_panel {
            SidePanel::left("audio_file_panel")
                .frame(Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                .default_width(200.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                    self.ui_audio_files.ui(ui, Some(frame), &mut self.state);
                });
        }

        if self.show_point_edit_panel {
            SidePanel::left("point_edit_panel")
                .frame(Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                .default_width(200.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                    self.ui_point_edit
                        .ui(ui, &self.state.audio_entries, &mut self.state.selection);
                });
        }

        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                self.ui_pitch_volume_plots.ui(ui, &mut self.state);
            });
    }
}
