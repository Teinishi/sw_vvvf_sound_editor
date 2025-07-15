use crate::{
    state::State,
    ui::{UiAudioFiles, UiFunctionEdit, aixs_hint_formatter_percentage},
};
use egui_extras::{Size, StripBuilder};
use egui_plot::{AxisHints, Plot};

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct MainApp {
    state: State,
    #[serde(skip)]
    ui_audio_files: UiAudioFiles,
    #[serde(skip)]
    ui_pitch_plot: UiFunctionEdit,
    #[serde(skip)]
    ui_volume_plot: UiFunctionEdit,
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
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

        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(200.0))
                .size(Size::remainder())
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        self.ui_audio_files.ui(ui, Some(frame), &mut self.state);
                    });

                    strip.cell(|ui| {
                        let height = ui.available_height();

                        let mut selection = self.state.selection.clone();
                        self.ui_pitch_plot.ui(
                            ui,
                            &mut self.state.pitch_entries_mut(),
                            &mut selection,
                            || {
                                Plot::new("plot_edit_volume")
                                    .show_axes(true)
                                    .show_grid(true)
                                    .default_x_bounds(0.0, 100.0)
                                    .default_y_bounds(0.0, 3.0)
                                    .custom_x_axes(vec![])
                                    .custom_y_axes(vec![AxisHints::new_y().label("Pitch")])
                                    .height(height / 2.0)
                            },
                            |_| {},
                        );
                        self.ui_volume_plot.ui(
                            ui,
                            &mut self.state.volume_entries_mut(),
                            &mut selection,
                            || {
                                Plot::new("plot_edit_pitch")
                                    .show_axes(true)
                                    .show_grid(true)
                                    .default_x_bounds(0.0, 100.0)
                                    .default_y_bounds(0.0, 1.1)
                                    .custom_x_axes(vec![AxisHints::new_x().label("Speed")])
                                    .custom_y_axes(vec![
                                        AxisHints::new_y()
                                            .label("Volume")
                                            .formatter(aixs_hint_formatter_percentage),
                                    ])
                            },
                            |_| {},
                        );
                        self.state.selection = selection;
                    });
                });
        });
    }
}
