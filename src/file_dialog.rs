use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rfd::FileDialog;
use std::path::PathBuf;

fn dialog_with_parent<W: HasWindowHandle + HasDisplayHandle>(parent: Option<&W>) -> FileDialog {
    let mut dialog = rfd::FileDialog::new();
    if let Some(p) = parent {
        dialog = dialog.set_parent(p);
    }
    dialog
}

fn filter_project(dialog: FileDialog) -> FileDialog {
    dialog.add_filter("SW VVVF Project", &["swvf"])
}

fn filter_json(dialog: FileDialog) -> FileDialog {
    dialog.add_filter("JSON File", &["json"])
}

fn filter_ogg(dialog: FileDialog) -> FileDialog {
    dialog.add_filter("Ogg Vorbis File", &["ogg"])
}

pub fn save_project_dialog<W: HasWindowHandle + HasDisplayHandle>(
    parent: Option<&W>,
) -> Option<PathBuf> {
    filter_project(dialog_with_parent(parent)).save_file()
}

pub fn open_json_dialog<W: HasWindowHandle + HasDisplayHandle>(
    parent: Option<&W>,
) -> Option<PathBuf> {
    filter_json(dialog_with_parent(parent)).pick_file()
}

pub fn add_ogg_dialog<W: HasWindowHandle + HasDisplayHandle>(
    parent: Option<&W>,
) -> Option<Vec<PathBuf>> {
    filter_ogg(dialog_with_parent(parent)).pick_files()
}
