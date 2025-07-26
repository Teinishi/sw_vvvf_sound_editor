use crate::{
    app_action::AppAction,
    audio_player::AudioOutput,
    player_state::PlayerState,
    preference::Preference,
    state::{FileRegistory, State},
    ui::{
        UiAudioFiles, UiMenuBar, UiPerformanceWindow, UiPitchVolumeEdit, UiPitchVolumePlots,
        UiPlayer, UiSettingWindow,
    },
};
use egui::{
    CentralPanel, Frame, ScrollArea, SidePanel, TopBottomPanel, util::undoer::Undoer, vec2,
};
use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MainApp {
    #[serde(skip)]
    registory: FileRegistory,
    #[serde(skip)]
    audio_output: AudioOutput,
    #[serde(skip)]
    undoer: Undoer<State>,
    state: State,
    state_file: Option<PathBuf>,
    preference: Preference,
    #[serde(skip)]
    player_state: PlayerState,
    ui_menu_bar: UiMenuBar,
    #[serde(skip)]
    ui_audio_files: UiAudioFiles,
    #[serde(skip)]
    ui_point_edit: UiPitchVolumeEdit,
    #[serde(skip)]
    ui_pitch_volume_plots: UiPitchVolumePlots,
    #[serde(skip)]
    ui_player: UiPlayer,
    #[serde(skip)]
    ui_performance_window: UiPerformanceWindow,
    #[serde(skip)]
    ui_setting_window: UiSettingWindow,
    #[serde(skip)]
    action: AppAction,
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            registory: FileRegistory::default(),
            audio_output: AudioOutput::default(),
            undoer: Undoer::with_settings(egui::util::undoer::Settings {
                stable_time: 0.5,
                ..Default::default()
            }),
            state: State::default(),
            state_file: None,
            preference: Preference::default(),
            player_state: PlayerState::default(),
            ui_menu_bar: UiMenuBar::default(),
            ui_audio_files: UiAudioFiles,
            ui_point_edit: UiPitchVolumeEdit,
            ui_pitch_volume_plots: UiPitchVolumePlots::default(),
            ui_player: UiPlayer,
            ui_performance_window: UiPerformanceWindow::default(),
            ui_setting_window: UiSettingWindow,
            action: AppAction::default(),
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
        let mut s: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        s.init();
        s
    }

    fn init(&mut self) {
        self.player_state.check(&self.state.train_performance);
        self.registory
            .play_audio(&mut self.audio_output)
            .expect("Failed to play audio.");
    }

    fn ui(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.ui_menu_bar
                .ui(ui, &mut self.action, self.state_file.is_some());
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

        if self.ui_menu_bar.show_audio_files_panel {
            SidePanel::left("audio_file_panel")
                .frame(Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                .default_width(190.0)
                .min_width(120.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                    self.ui_audio_files.ui(
                        ui,
                        Some(frame),
                        &mut self.action,
                        &mut self.registory,
                        &mut self.state,
                    );
                });
        }

        if self.ui_menu_bar.show_point_edit_panel {
            SidePanel::left("point_edit_panel")
                .frame(Frame::side_top_panel(&ctx.style()).inner_margin(8.0))
                .default_width(190.0)
                .min_width(190.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                        self.ui_point_edit.ui(
                            ui,
                            &mut self.action,
                            &mut self.state.audio_entries,
                            &mut self.state.selection,
                        );
                    });
                });
        }

        TopBottomPanel::bottom("train_speed_control")
            .exact_height(80.0)
            .show(ctx, |ui| {
                self.ui_player
                    .ui(ui, &mut self.preference, &mut self.player_state);
            });

        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                self.ui_pitch_volume_plots.ui(
                    ui,
                    &mut self.action,
                    &mut self.state,
                    &self.player_state,
                );
            });

        // ウィンドウ
        self.ui_performance_window.show(
            ctx,
            &mut self.ui_menu_bar.show_performance_window,
            &mut self.action,
            &mut self.state.train_performance,
        );
        self.ui_setting_window.show(
            ctx,
            &mut self.ui_menu_bar.show_setting_window,
            &mut self.state,
        );
    }
}

impl eframe::App for MainApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.action.new_frame(&self.undoer, &self.state);

        self.ui(ctx, frame);

        // 毎フレームの更新処理
        self.state.train_performance.update();
        self.player_state.update(ctx, &self.state, &self.preference);
        self.registory.update(
            &mut self.state,
            &self.player_state,
            &self.preference,
            &mut self.action,
        );

        // action を実行、エラーをモーダルで表示
        self.action.shortcut(ctx);
        self.action.exec(
            ctx,
            Some(frame),
            &mut self.registory,
            &mut self.state,
            &mut self.undoer,
            &mut self.state_file,
        );
        self.action.show_modal(ctx);

        // undoer 更新
        self.undoer.feed_state(ctx.input(|i| i.time), &self.state);
    }
}
