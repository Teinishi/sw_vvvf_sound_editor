use super::{UiPlotEdit, aixs_hint_formatter_percentage};
use crate::{
    app::AppAction, player_state::PlayerState, state::{AudioEntryId, State}, ui::ui_plot_edit::PlotEditEntry,
};
use egui::{Color32, Sides};
use egui_plot::{AxisHints, Plot, VLine};
use std::{collections::HashMap, hash::Hash, ops::RangeInclusive};

#[derive(Debug)]
struct PlotLink<K: Hash> {
    last_bound: HashMap<K, (RangeInclusive<f64>, RangeInclusive<f64>)>,
    priority: Option<K>,
    enable_x: bool,
    enable_y: bool,
}

impl<K: Hash> Default for PlotLink<K> {
    fn default() -> Self {
        Self {
            last_bound: HashMap::new(),
            priority: None,
            enable_x: true,
            enable_y: true,
        }
    }
}

impl<K: Hash> PlotLink<K> {
    fn new_x() -> Self {
        Self {
            enable_y: false,
            ..Default::default()
        }
    }

    #[expect(unused)]
    fn new_y() -> Self {
        Self {
            enable_x: false,
            ..Default::default()
        }
    }

    fn update(&mut self, plot_ui: &mut egui_plot::PlotUi<'_>, variant: K)
    where
        K: Eq + Clone + std::fmt::Debug,
    {
        let b = plot_ui.plot_bounds();
        let mut range_x = b.range_x();
        let mut range_y = b.range_y();

        if let Some(last) = &self.last_bound.get(&variant) {
            let changed_x = self.enable_x && last.0 != range_x;
            let changed_y = self.enable_y && last.1 != range_y;
            if changed_x || changed_y {
                self.priority = Some(variant.clone());
            } else if let Some((prange_x, prange_y)) = self
                .priority
                .as_ref()
                .and_then(|p| (p != &variant).then(|| self.last_bound.get(p)).flatten())
            {
                if self.enable_x && prange_x != &range_x {
                    plot_ui.set_plot_bounds_x(prange_x.clone());
                    range_x = prange_x.clone();
                }
                if self.enable_y && prange_y != &range_y {
                    plot_ui.set_plot_bounds_y(prange_y.clone());
                    range_y = prange_y.clone();
                }
            }
        }

        self.last_bound.insert(variant, (range_x, range_y));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PlotVariant {
    Pitch,
    Volume,
}

#[derive(Debug)]
pub struct UiPitchVolumePlots {
    ui_pitch_plot: UiPlotEdit<AudioEntryId>,
    ui_volume_plot: UiPlotEdit<AudioEntryId>,
    plot_link: PlotLink<PlotVariant>,
}

impl Default for UiPitchVolumePlots {
    fn default() -> Self {
        Self {
            ui_pitch_plot: Default::default(),
            ui_volume_plot: Default::default(),
            plot_link: PlotLink::new_x(),
        }
    }
}

impl UiPitchVolumePlots {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        action: &mut AppAction,
        state: &mut State,
        player_state: &PlayerState,
    ) {
        let height = ui.available_height();
        let speed_line_color = match ui.ctx().theme() {
            egui::Theme::Light => Color32::DARK_GRAY,
            egui::Theme::Dark => Color32::LIGHT_GRAY,
        };

        let mut reset_viewport = false;

        Sides::new().show(
            ui,
            |_| {},
            |ui| {
                reset_viewport = ui.button("Reset viewport").clicked();
            },
        );

        self.ui_pitch_plot.ui(
            ui,
            action,
            &mut PlotEditEntry::pitch(&mut state.audio_entries, &state.selection),
            &mut state.selection,
            &mut None,
            || {
                Plot::new("plot_edit_volume")
                    .show_axes(true)
                    .show_grid(true)
                    .default_x_bounds(0.0, 120.0)
                    .default_y_bounds(0.0, 3.0)
                    .custom_x_axes(vec![])
                    .custom_y_axes(vec![AxisHints::new_y().label("Pitch").min_thickness(60.0)])
                    .height(height / 2.0 - 18.0)
            },
            |plot_ui: &mut egui_plot::PlotUi<'_>| {
                plot_ui.vline(
                    VLine::new("", player_state.speed)
                        .allow_hover(false)
                        .color(speed_line_color),
                );

                if reset_viewport {
                    plot_ui.set_auto_bounds(true);
                } else {
                    self.plot_link.update(plot_ui, PlotVariant::Pitch);
                }
            },
        );

        self.ui_volume_plot.ui(
            ui,
            action,
            &mut PlotEditEntry::volume(&mut state.audio_entries),
            &mut state.selection,
            &mut None,
            || {
                Plot::new("plot_edit_pitch")
                    .show_axes(true)
                    .show_grid(true)
                    .default_x_bounds(0.0, 120.0)
                    .default_y_bounds(0.0, 1.1)
                    .custom_x_axes(vec![AxisHints::new_x().label("Speed (km/h)")])
                    .custom_y_axes(vec![
                        AxisHints::new_y()
                            .label("Volume")
                            .min_thickness(60.0)
                            .formatter(aixs_hint_formatter_percentage),
                    ])
            },
            |plot_ui: &mut egui_plot::PlotUi<'_>| {
                plot_ui.vline(
                    VLine::new("", player_state.speed)
                        .allow_hover(false)
                        .color(speed_line_color),
                );

                if reset_viewport {
                    plot_ui.set_auto_bounds(true);
                } else {
                    self.plot_link.update(plot_ui, PlotVariant::Volume);
                }
            },
        );
    }
}
