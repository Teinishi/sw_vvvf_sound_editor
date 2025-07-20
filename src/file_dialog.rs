use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use rfd::FileDialog;

fn dialog_with_parent<W: HasWindowHandle + HasDisplayHandle>(parent: Option<&W>) -> FileDialog {
    let mut dialog = rfd::FileDialog::new();
    if let Some(p) = parent {
        dialog = dialog.set_parent(p);
    }
    dialog
}

fn filter_json(dialog: FileDialog) -> FileDialog {
    dialog.add_filter("JSON File", &["json"])
}

pub fn save_json_dialog<W: HasWindowHandle + HasDisplayHandle>(
    parent: Option<&W>,
) -> Option<std::path::PathBuf> {
    filter_json(dialog_with_parent(parent)).save_file()
}

pub fn open_json_dialog<W: HasWindowHandle + HasDisplayHandle>(
    parent: Option<&W>,
) -> Option<std::path::PathBuf> {
    filter_json(dialog_with_parent(parent)).pick_file()
}
