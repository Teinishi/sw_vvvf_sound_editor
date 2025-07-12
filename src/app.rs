use crate::ui::UiAudioFiles;
use std::path::PathBuf;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct MainApp {
    work_folder: Option<PathBuf>,
    ui_audio_files: UiAudioFiles,
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

    #[cfg(not(target_arch = "wasm32"))]
    fn open_folder<W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle>(
        &mut self,
        parent: Option<&W>,
    ) -> std::io::Result<()> {
        let mut dialog = rfd::FileDialog::new();
        if let Some(p) = parent {
            dialog = dialog.set_parent(p);
        }
        if let Some(pathbuf) = dialog.pick_folder() {
            self.read_folder(&pathbuf)?;
            self.work_folder = Some(pathbuf);
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn read_folder<P: AsRef<std::path::Path>>(&mut self, path: P) -> std::io::Result<()> {
        self.ui_audio_files.clear();

        for entry in (std::fs::read_dir(path)?).flatten() {
            let entry_path = entry.path();
            if entry_path.is_file()
                && entry_path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ogg"))
            {
                self.ui_audio_files.add_inactive_audio_file(entry_path);
            }
        }

        Ok(())
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
                        if ui.button("Open folder").clicked() {
                            if let Err(err) = self.open_folder(Some(frame)) {
                                eprintln!("{err:?}");
                            }
                            ui.close();
                        }

                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        if let Some(path) = &self.work_folder {
            if let Some(path_str) = path.to_str() {
                egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                    ui.add_space(4.0);
                    ui.weak(path_str);
                    ui.add_space(2.0);
                });
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_audio_files.ui(ui);
        });
    }
}
