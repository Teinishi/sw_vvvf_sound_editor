use super::{PlotAutoColor, UiPlotEdit};
use crate::{app::AppAction, state::State, ui::UiFunctionEdit};
use egui::{ScrollArea, Sides, Window};
use egui_plot::{AxisHints, Plot};

#[derive(Debug, Default)]
pub struct UiPerformanceWindow {
    acceleration_plot: UiPlotEdit,
    selection: Option<()>,
}

impl UiPerformanceWindow {
    pub const TITLE: &str = "Train Performance";

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        action: &mut AppAction,
        state: &mut State,
    ) {
        Window::new(Self::TITLE)
            .open(open)
            .default_size([500.0, 400.0])
            .min_size([300.0, 200.0])
            .show(ctx, |ui| {
                self.ui(ui, action, state);
            });
    }

    fn ui(&mut self, ui: &mut egui::Ui, action: &mut AppAction, state: &mut State) {
        ScrollArea::vertical().show(ui, |ui| {
            let mut reset_viewport = false;

            Sides::new().show(
                ui,
                |ui| {
                    ui.strong("Speed - Acceleration");
                },
                |ui| {
                    reset_viewport = ui.button("Reset viewport").clicked();
                },
            );

            let mut entries = vec![(&mut state.acceleration, PlotAutoColor::get_color(0), ())];
            self.acceleration_plot.ui(
                ui,
                action,
                &mut entries,
                &mut self.selection,
                &mut None,
                || {
                    Plot::new("plot_edit_acceleration")
                        .show_axes(true)
                        .show_grid(true)
                        .default_x_bounds(0.0, 150.0)
                        .default_y_bounds(0.0, 4.0)
                        .custom_x_axes(vec![AxisHints::new_x().label("Speed (km/h)")])
                        .custom_y_axes(vec![AxisHints::new_y().label("Acceleration (km/h/s)")])
                        .height(200.0)
                },
                |plot_ui: &mut egui_plot::PlotUi<'_>| {
                    if reset_viewport {
                        plot_ui.set_auto_bounds(true);
                    }
                },
            );

            UiFunctionEdit::new("Accleration", ("Speed", "Acceleration"))
                .ui(ui, &mut state.acceleration);
        });
    }
}
