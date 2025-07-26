use crate::state::{FileRegistory, State};
use egui::{Id, Key, KeyboardShortcut, Modal, Modifiers, Sides, util::undoer::Undoer};
use std::{collections::VecDeque, path::PathBuf};

#[derive(Debug, Default)]
pub struct AppAction {
    inner: AppActionInner,
    modals: AppModals,
}

impl AppAction {
    pub fn new_frame(&mut self, undoer: &Undoer<State>, state: &State) {
        self.inner = AppActionInner::new(undoer, state);
    }

    pub fn shortcut(&mut self, ctx: &egui::Context) {
        let (ctrl_n, ctrl_o, ctrl_s, ctrl_shfit_s) = ctx.input_mut(|i| {
            (
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::N)),
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::O)),
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::S)),
                i.consume_shortcut(&KeyboardShortcut::new(
                    Modifiers::COMMAND.plus(Modifiers::SHIFT),
                    Key::S,
                )),
            )
        });
        if ctrl_n {
            self.new_project();
        }
        if ctrl_o {
            self.open();
        }
        if ctrl_s {
            self.save();
        }
        if ctrl_shfit_s {
            self.save_as();
        }
    }

    pub fn exec<W>(
        &mut self,
        ctx: &egui::Context,
        parent: Option<&W>,
        registory: &mut FileRegistory,
        state: &mut State,
        undoer: &mut Undoer<State>,
        state_filepath: &mut Option<PathBuf>,
    ) where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        if let Err(err) = self
            .inner
            .exec(ctx, parent, registory, state, undoer, state_filepath)
        {
            self.add_error_modal(err);
        }
    }

    pub fn has_undo(&self) -> bool {
        self.inner.has_undo
    }

    pub fn has_redo(&self) -> bool {
        self.inner.has_redo
    }

    pub fn new_project(&mut self) {
        self.inner.new_project = true;
    }

    pub fn open(&mut self) {
        self.inner.open = true;
    }

    pub fn save(&mut self) {
        self.inner.save = true;
    }

    pub fn save_as(&mut self) {
        self.inner.save_as = true;
    }

    pub fn quit(&mut self) {
        self.inner.quit = true;
    }

    pub fn add_undo(&mut self) {
        self.inner.add_undo = true;
    }

    pub fn undo(&mut self) {
        self.inner.undo = true;
    }

    pub fn redo(&mut self) {
        self.inner.redo = true;
    }

    pub fn add_error_modal(&mut self, error: anyhow::Error) {
        self.modals.errors.push(error);
    }

    pub fn add_confirmation_modal(&mut self, message: String) {
        self.modals.confirmations.push_back(message);
    }

    pub fn show_modal(&mut self, ctx: &egui::Context) {
        self.modals.show(ctx);
    }
}

#[derive(Debug, Default)]
struct AppModals {
    errors: Vec<anyhow::Error>,
    confirmations: VecDeque<String>,
}

impl AppModals {
    fn show(&mut self, ctx: &egui::Context) {
        if !self.errors.is_empty() {
            let modal = Modal::new(Id::new("modal_error")).show(ctx, |ui| {
                ui.heading("\u{26a0} Error");
                for error in &self.errors {
                    ui.add_space(8.0);
                    ui.label(format!("{error:?}"));
                }
                ui.add_space(8.0);
                Sides::new()
                    .show(ui, |_| {}, |ui| ui.button("Close").clicked())
                    .1
            });
            if modal.should_close() || modal.inner {
                self.errors.clear();
            }
        }
    }
}

#[derive(Debug, Default)]
struct AppActionInner {
    has_undo: bool,
    has_redo: bool,

    new_project: bool,
    open: bool,
    save: bool,
    save_as: bool,
    quit: bool,
    add_undo: bool,
    undo: bool,
    redo: bool,
}

impl AppActionInner {
    fn new<T: Clone + PartialEq>(undoer: &Undoer<T>, state: &T) -> Self {
        Self {
            has_undo: undoer.has_undo(state),
            has_redo: undoer.has_redo(state),
            ..Default::default()
        }
    }

    fn exec<W>(
        &self,
        ctx: &egui::Context,
        parent: Option<&W>,
        registory: &mut FileRegistory,
        state: &mut State,
        undoer: &mut Undoer<State>,
        state_filepath: &mut Option<PathBuf>,
    ) -> anyhow::Result<()>
    where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        // エラーにならないものを先にやっておく
        if self.quit {
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
        if self.new_project {
            // todo: 確認ダイアログ
            *state = State::default();
        }

        // エラーになりうるもの
        #[cfg(not(target_arch = "wasm32"))]
        if self.open {
            if let Some(path) = crate::file_dialog::open_project_dialog(parent) {
                crate::save_load::open_file(path, registory, state, state_filepath)?;
            }
        }
        let mut save_as = self.save_as;
        #[cfg(not(target_arch = "wasm32"))]
        if self.save {
            if let Some(path) = &state_filepath {
                crate::save_load::save_file(path.clone(), registory, state, state_filepath)?;
            } else {
                save_as = true;
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if save_as {
            if let Some(path) = crate::file_dialog::save_project_dialog(parent) {
                crate::save_load::save_file(path, registory, state, state_filepath)?;
            }
        }

        Ok(())
    }
}
