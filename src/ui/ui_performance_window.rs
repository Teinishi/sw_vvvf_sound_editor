use egui::Window;

#[derive(Debug, Default)]
pub struct UiPerformanceWindow;

impl UiPerformanceWindow {
    pub const TITLE: &str = "Train Performance";

    pub fn show(&self, ctx: &egui::Context, open: &mut bool) {
        Window::new(Self::TITLE)
            .open(open)
            .default_size([300.0, 200.0])
            .min_size([200.0, 100.0])
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }

    #[expect(clippy::unused_self)]
    fn ui(&self, ui: &mut egui::Ui) {
        ui.label(Self::TITLE);
    }
}
