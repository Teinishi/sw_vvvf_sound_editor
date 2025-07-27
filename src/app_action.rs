use crate::{
    app::StateFilePath,
    state::{FileRegistory, State},
};
use egui::{Id, Key, KeyboardShortcut, Modal, Modifiers, Sides, util::undoer::Undoer};
use std::{collections::VecDeque, path::PathBuf};

#[derive(Debug, Default)]
pub struct AppAction {
    has_undo: bool,
    has_redo: bool,
    flags: AppActionFlags,
    modals: AppModals,
}

impl AppAction {
    pub fn new_frame(&mut self, undoer: &Undoer<State>, state: &State) {
        self.has_undo = undoer.has_undo(state);
        self.has_redo = undoer.has_redo(state);
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
        state_filepath: &mut Option<StateFilePath>,
    ) where
        W: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle,
    {
        // エラーにならないものを先にやっておく
        if self.flags.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.flags.add_undo {
            undoer.add_undo(state);
        }
        if self.flags.undo
            || ctx.input_mut(|i| {
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Z))
            })
        {
            if let Some(s) = undoer.undo(state) {
                *state = s.clone();
            }
        }
        if self.flags.redo
            || ctx.input_mut(|i| {
                i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Y))
            })
        {
            if let Some(s) = undoer.redo(state) {
                *state = s.clone();
            }
        }
        if self.flags.new_project {
            self.add_confirmation_modal(
                "Are you sure you want to create a new project? This will discard unsaved changes."
                    .to_owned(),
                AppActionFlags {
                    new_project_confirmed: true,
                    ..Default::default()
                },
            );
        }
        if self.flags.new_project_confirmed {
            registory.clear(state);
            *state_filepath = None;
        }

        // エラーになりうるもの
        #[cfg(not(target_arch = "wasm32"))]
        if self.flags.open {
            if let Some(path) = crate::file_dialog::open_project_dialog(parent) {
                let result = crate::save_load::load_file(&path, registory, state);
                match result {
                    Ok(_) => {
                        *state_filepath = Some(StateFilePath::new(path, state.clone()));
                    }
                    Err(err) => {
                        self.add_error_modal(err);
                    }
                }
            }
        }
        let mut save_as = self.flags.save_as;
        #[cfg(not(target_arch = "wasm32"))]
        if self.flags.save {
            if let Some(StateFilePath { path, .. }) = &state_filepath {
                self.do_save(path.clone(), registory, state, state_filepath);
            } else {
                save_as = true;
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if save_as {
            if let Some(path) = crate::file_dialog::save_project_dialog(parent) {
                self.do_save(path, registory, state, state_filepath);
            }
        }

        self.flags = AppActionFlags::default();
    }

    pub fn has_undo(&self) -> bool {
        self.has_undo
    }

    pub fn has_redo(&self) -> bool {
        self.has_redo
    }

    pub fn new_project(&mut self) {
        self.flags.new_project = true;
    }

    pub fn open(&mut self) {
        self.flags.open = true;
    }

    pub fn save(&mut self) {
        self.flags.save = true;
    }

    pub fn save_as(&mut self) {
        self.flags.save_as = true;
    }

    pub fn quit(&mut self) {
        self.flags.quit = true;
    }

    pub fn add_undo(&mut self) {
        self.flags.add_undo = true;
    }

    pub fn undo(&mut self) {
        self.flags.undo = true;
    }

    pub fn redo(&mut self) {
        self.flags.redo = true;
    }

    pub fn add_error_modal(&mut self, error: anyhow::Error) {
        self.modals.errors.push(error);
    }

    fn add_confirmation_modal(&mut self, message: String, flags: AppActionFlags) {
        self.modals.confirmations.push_back((message, flags));
    }

    pub fn show_modal(&mut self, ctx: &egui::Context) {
        self.modals.show(ctx, &mut self.flags);
    }

    fn do_save(
        &mut self,
        path: PathBuf,
        registory: &FileRegistory,
        state: &State,
        state_filepath: &mut Option<StateFilePath>,
    ) {
        let result = crate::save_load::save_file(&path, registory, state);
        match result {
            Ok(_) => {
                *state_filepath = Some(StateFilePath::new(path, state.clone()));
            }
            Err(err) => {
                self.add_error_modal(err);
            }
        }
    }
}

#[derive(Debug, Default)]
struct AppModals {
    errors: Vec<anyhow::Error>,
    confirmations: VecDeque<(String, AppActionFlags)>,
}

impl AppModals {
    fn show(&mut self, ctx: &egui::Context, flags: &mut AppActionFlags) {
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
        } else if let Some((message, modal_flags)) = self.confirmations.front() {
            let modal = Modal::new(Id::new("modal_confirmation")).show(ctx, |ui| {
                ui.heading("\u{2757} Confirmation");
                ui.add_space(8.0);
                ui.label(message);
                ui.add_space(8.0);
                Sides::new()
                    .show(
                        ui,
                        |_| {},
                        |ui| (ui.button("OK").clicked(), ui.button("Cancel").clicked()),
                    )
                    .1
            });
            if modal.inner.0 {
                // OK was clicked
                *flags |= *modal_flags;
            }
            if modal.should_close() || modal.inner.0 || modal.inner.1 {
                self.confirmations.pop_front();
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct AppActionFlags {
    new_project: bool,
    new_project_confirmed: bool,
    open: bool,
    save: bool,
    save_as: bool,
    quit: bool,
    add_undo: bool,
    undo: bool,
    redo: bool,
}

impl std::ops::BitOr for AppActionFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            new_project: self.new_project || rhs.new_project,
            new_project_confirmed: self.new_project_confirmed || rhs.new_project_confirmed,
            open: self.open || rhs.open,
            save: self.save || rhs.save,
            save_as: self.save_as || rhs.save_as,
            quit: self.quit || rhs.quit,
            add_undo: self.add_undo || rhs.add_undo,
            undo: self.undo || rhs.undo,
            redo: self.redo || rhs.redo,
        }
    }
}

impl std::ops::BitOrAssign for AppActionFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
