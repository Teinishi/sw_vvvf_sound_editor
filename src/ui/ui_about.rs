pub fn ui_about_rev(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.hyperlink_to(
            "\u{E624} GitHub repository",
            "https://github.com/Teinishi/sw_vvvf_sound_editor",
        );
        ui.hyperlink_to(
            "MIT License",
            "https://github.com/Teinishi/sw_vvvf_sound_editor/blob/main/LICENSE-MIT",
        );
        ui.hyperlink_to(
            "Apache-2.0",
            "https://github.com/Teinishi/sw_vvvf_sound_editor/blob/main/LICENSE-APACHE",
        );
    });

    ui.horizontal(|ui| {
        ui.label(env!("CARGO_PKG_NAME"));
        ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
        ui.label("\u{A9} 2025 Teinishi");
    });
}
