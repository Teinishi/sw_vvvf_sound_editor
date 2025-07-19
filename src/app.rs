use crate::{
    player_state::PlayerState,
    state::State,
    ui::{
        UiAudioFiles, UiMenuBar, UiPerformanceWindow, UiPitchVolumeEdit, UiPitchVolumePlots,
        UiPlayer, UiSettingWindow,
    },
};
use egui::{
    CentralPanel, Frame, Key, KeyboardShortcut, Modifiers, ScrollArea, SidePanel, TopBottomPanel,
    util::undoer::Undoer, vec2,
};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MainApp {
    #[serde(skip)]
    undoer: Undoer<State>,
    state: State,
    #[serde(skip)]
    player_state: PlayerState,
    show_audio_files_panel: bool,
    show_point_edit_panel: bool,
    show_performance_window: bool,
    show_setting_window: bool,
    #[serde(skip)]
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
}

impl Default for MainApp {
    fn default() -> Self {
        Self {
            undoer: Undoer::with_settings(egui::util::undoer::Settings {
                stable_time: 0.5,
                ..Default::default()
            }),
            state: State::default(),
            player_state: PlayerState::default(),
            show_audio_files_panel: true,
            show_point_edit_panel: true,
            show_performance_window: false,
            show_setting_window: false,
            ui_menu_bar: UiMenuBar,
            ui_audio_files: UiAudioFiles,
            ui_point_edit: UiPitchVolumeEdit,
            ui_pitch_volume_plots: UiPitchVolumePlots::default(),
            ui_player: UiPlayer,
            ui_performance_window: UiPerformanceWindow::default(),
            ui_setting_window: UiSettingWindow,
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
        self.player_state.play().expect("Failed to play audio.");
    }
}

impl eframe::App for MainApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut action = AppAction::new(&self.undoer, &self.state);

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.ui_menu_bar.ui(
                ui,
                &mut action,
                &mut self.show_audio_files_panel,
                &mut self.show_point_edit_panel,
                &mut self.show_performance_window,
                &mut self.show_setting_window,
            );
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
                .default_width(190.0)
                .min_width(120.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                    self.ui_audio_files
                        .ui(ui, Some(frame), &mut action, &mut self.state);
                });
        }

        if self.show_point_edit_panel {
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
                            &mut action,
                            &mut self.state.audio_entries,
                            &mut self.state.selection,
                        );
                    });
                });
        }

        TopBottomPanel::bottom("train_speed_control")
            .exact_height(100.0)
            .show(ctx, |ui| {
                self.ui_player.ui(ui, &mut self.player_state);
            });

        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = vec2(8.0, 8.0);
                self.ui_pitch_volume_plots
                    .ui(ui, &mut action, &mut self.state, &self.player_state);
            });

        // ウィンドウ
        self.ui_performance_window.show(
            ctx,
            &mut self.show_performance_window,
            &mut action,
            &mut self.state.train_performance,
        );
        self.ui_setting_window
            .show(ctx, &mut self.show_setting_window, &mut self.state);

        self.state.train_performance.update();
        self.player_state.update(ctx, &self.state.train_performance);

        action.exec(ctx, &mut self.state, &mut self.undoer);

        self.undoer.feed_state(ctx.input(|i| i.time), &self.state);
    }
}

#[derive(Debug, Default, Clone)]
pub struct AppAction {
    has_undo: bool,
    has_redo: bool,

    close: bool,
    add_undo: bool,
    undo: bool,
    redo: bool,
}

impl AppAction {
    fn new<T: Clone + PartialEq>(undoer: &Undoer<T>, state: &T) -> Self {
        Self {
            has_undo: undoer.has_undo(state),
            has_redo: undoer.has_redo(state),
            ..Default::default()
        }
    }

    fn exec<T: Clone + PartialEq>(
        &self,
        ctx: &egui::Context,
        state: &mut T,
        undoer: &mut Undoer<T>,
    ) {
        if self.close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.add_undo {
            undoer.add_undo(state);
        }
        if self.undo
            || ctx.input_mut(|i| {
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Z))
            })
        {
            if let Some(s) = undoer.undo(state) {
                *state = s.clone();
            }
        }
        if self.redo
            || ctx.input_mut(|i| {
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Y))
            })
        {
            if let Some(s) = undoer.redo(state) {
                *state = s.clone();
            }
        }
    }

    pub fn has_undo(&self) -> bool {
        self.has_undo
    }

    pub fn has_redo(&self) -> bool {
        self.has_redo
    }

    pub fn close(&mut self) {
        self.close = true;
    }

    pub fn add_undo(&mut self) {
        self.add_undo = true;
    }

    pub fn undo(&mut self) {
        self.undo = true;
    }

    pub fn redo(&mut self) {
        self.redo = true;
    }
}
