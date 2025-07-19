use crate::editable_function::{EditableFunction, EditableFunctionMode};
use egui::{Button, ComboBox, DragValue, Label, Popup, RichText, vec2};
use egui_extras::{Column, TableBuilder};

#[derive(Debug, Default)]
pub struct UiFunctionEdit<'a> {
    title: &'a str,
    axis_label: (&'a str, &'a str),
    percentage: (bool, bool),
}

impl<'a> UiFunctionEdit<'a> {
    pub fn new(title: &'a str, axis_label: (&'a str, &'a str)) -> Self {
        Self {
            title,
            axis_label,
            percentage: (false, false),
        }
    }

    #[expect(dead_code)]
    pub fn x_percentage(mut self, value: bool) -> Self {
        self.percentage.0 = value;
        self
    }

    pub fn y_percentage(mut self, value: bool) -> Self {
        self.percentage.1 = value;
        self
    }

    pub fn ui(
        &self,
        ui: &mut egui::Ui,
        id_salt: impl std::hash::Hash,
        func: &mut EditableFunction,
    ) {
        ui.horizontal(|ui| {
            ui.label(self.title);
            ComboBox::new(id_salt, "")
                .selected_text(format!("{:?}", func.mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut func.mode, EditableFunctionMode::Points, "Points");
                    ui.selectable_value(
                        &mut func.mode,
                        EditableFunctionMode::Expression,
                        "Expression",
                    );
                });
        });

        match func.mode {
            EditableFunctionMode::Points => {
                self.ui_points(ui, func);
            }
            EditableFunctionMode::Expression => {
                self.ui_expression(ui, func);
            }
        }
    }

    fn ui_points(&self, ui: &mut egui::Ui, func: &mut EditableFunction) {
        let mut update_point = None;
        let mut remove_point = None;
        let mut add_point = None;

        TableBuilder::new(ui)
            .column(Column::initial(16.0))
            .columns(Column::exact(60.0), 2)
            .column(Column::exact(20.0))
            .vscroll(false)
            .header(20.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(self.axis_label.0);
                    });
                });
                header.col(|ui| {
                    ui.centered_and_justified(|ui| {
                        ui.label(self.axis_label.1);
                    });
                });
                header.col(|_| {});
            })
            .body(|mut body| {
                let points = func.points();
                let removable = points.len() > 1;
                for (i, point) in points.iter().enumerate() {
                    let mut x = point.0;
                    let mut y = point.1;

                    body.row(20.0, |mut row| {
                        // インデックス
                        row.col(|ui| {
                            ui.label(format!("{i}"));
                        });

                        // X座標
                        row.col(|ui| {
                            let mut drag_x = DragValue::new(&mut x).speed(0.1).max_decimals(1);
                            if self.percentage.0 {
                                drag_x = drag_x
                                    .custom_formatter(|x, _| format!("{:.1}", 100.0 * x))
                                    .custom_parser(|s| s.parse().ok().map(|v: f64| v / 100.0))
                                    .suffix("%");
                            }
                            if ui.add_sized(vec2(60.0, 20.0), drag_x).changed() {
                                update_point = Some((i, (x, y)));
                            }
                        });

                        // Y座標
                        row.col(|ui| {
                            let mut drag_y = DragValue::new(&mut y).speed(0.01).max_decimals(3);
                            if self.percentage.1 {
                                drag_y = drag_y
                                    .custom_formatter(|x, _| format!("{:.1}", 100.0 * x))
                                    .custom_parser(|s| s.parse().ok().map(|v: f64| v / 100.0))
                                    .suffix("%");
                            }
                            if ui.add_sized(vec2(60.0, 20.0), drag_y).changed() {
                                update_point = Some((i, (x, y)));
                            }
                        });

                        // 点追加・削除
                        row.col(|ui| {
                            let response = ui
                                .add_sized(ui.available_size(), Button::new("\u{b7}\u{b7}\u{b7}"));
                            Popup::menu(&response).show(|ui| {
                                if ui.button("+ Add before").clicked() {
                                    add_point = Some(i);
                                }
                                if ui.button("+ Add after").clicked() {
                                    add_point = Some(i + 1);
                                }
                                if ui.add_enabled(removable, Button::new("- Remove")).clicked() {
                                    remove_point = Some(i);
                                }
                            });
                        });
                    });
                }
            });

        if let Some((i, pos)) = update_point {
            func.move_point_to(i, pos);
        }
        if let Some(i) = remove_point {
            func.remove_point(i);
        }
        if let Some(i) = add_point {
            func.insert_point_by_index(i);
        }
    }

    #[expect(clippy::unused_self)]
    fn ui_expression(&self, ui: &mut egui::Ui, func: &mut EditableFunction) {
        ui.horizontal(|ui| {
            ui.label("f(x)=");
            ui.text_edit_singleline(&mut func.expression);
        });

        if let Some(err) = func.expression_err() {
            ui.add(Label::new(
                RichText::new(format!("{err}"))
                    .color(egui::Color32::RED)
                    .small(),
            ));
        }

        func.update();
    }
}
