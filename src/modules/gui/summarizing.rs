use crate::modules::financial::*;
use crate::modules::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use egui::{Align, ComboBox, Layout};
use egui_extras::*;
use strum::IntoEnumIterator;
use crate::modules::database::summaries::TimeUnit;

impl AppState {
    pub fn handle_show_expense_summary_window(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("expenses_summary_window"),
            egui::ViewportBuilder::default()
                .with_title("Expenses summary window")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    let expense_summary_csv = self.expense_summary_csv.clone();
                    let header_line: String = expense_summary_csv.split("\n").collect::<Vec<&str>>()[0].to_string();
                    let row_lines: Vec<&str> = expense_summary_csv.split("\n").collect::<Vec<&str>>()[1..].to_vec();
                    let column_count: usize = header_line.split(",").count();

                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::remainder().at_least(120.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                egui::Grid::new("expense_summary")
                                    .num_columns(3)
                                    .spacing([45.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Start date:").on_hover_text("Include expenses in the summary starting on the specified date, included.");
                                        ui.add(DatePickerButton::new(&mut self.expense_summary_date_from).id_salt("date_from"));
                                        ui.end_row();

                                        ui.label("End date:").on_hover_text("Include expenses in the summary until the specified date, included.");
                                        ui.add(DatePickerButton::new(&mut self.expense_summary_date_to).id_salt("date_to"));
                                        ui.end_row();

                                        ui.label("Currency:").on_hover_text("Currency in which to express the ammounts.");
                                        ComboBox::from_id_salt("Expense summary currency")
                                .selected_text(format!("{}", self.expense_summary_currency))
                                .show_ui(ui, |ui| {
                                    for possible_expense_summary_currency in Currency::iter() {
                                        ui.selectable_value(
                                            &mut self.expense_summary_currency,
                                            possible_expense_summary_currency.clone(),
                                            format!("{possible_expense_summary_currency}"),
                                        );
                                    }
                                });
                                        ui.end_row();

                                        ui.label("");
                                        if ui.button("Generate!").clicked() {
                                            match self.database.expenses_summary(
                                                self.expense_summary_date_from,
                                                self.expense_summary_date_to,
                                                &self.expense_summary_currency
                                            ) {
                                                Ok(s) => {self.expense_summary_csv = s; self.expense_summary_csv_correct = true;},
                                                Err(e) => {self.expense_summary_csv_correct = false; self.throw_error(e);}}
                                        }

                                    });
                                ui.separator();
                            });
                            if self.expense_summary_csv_correct {
                            strip.cell(|ui| {
                                                TableBuilder::new(ui)
                                        .columns(Column::auto().resizable(true), column_count)
                                        .striped(true)
                                        .cell_layout(Layout::right_to_left(Align::Center))
                                        .header(20.0, |mut header| {
                                            for column_name in header_line.split(",") {
                                                header.col(|ui| {
                                                    ui.strong(column_name)
                                                        .on_hover_text(column_name);
                                                });
                                            }
                                        })
                                        .body(|mut body| {
                                            for row_line in row_lines {
                                                body.row(30.0, |mut row_ui| {
                                                    let mut is_last_row: bool = false;
                                                    for element in row_line.split(",") {
                                                        if element == "Total" {
                                                            is_last_row = true;
                                                        }
                                                        row_ui.col(|ui| {
                                                            if is_last_row {
                                                                ui.strong(element);
                                                            } else {
                                                                ui.label(element);
                                                            }
                                                        });
                                                    }
                                                });
                                            }                         
                                        });
                                ui.separator();
                            });
                            }
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_expense_summary_window = false;
                }
            },
        )
    }
    pub fn handle_show_fund_stand_window(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("fund_stand_summary_window"),
            egui::ViewportBuilder::default()
                .with_title("Fund stand summary window")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    let fund_stand_csv = self.fund_stand_csv.clone();
                    let header_line: String = fund_stand_csv.split("\n").collect::<Vec<&str>>()[0].to_string();
                    let row_lines: Vec<&str> = fund_stand_csv.split("\n").collect::<Vec<&str>>()[1..].to_vec();
                    let column_count: usize = header_line.split(",").count();
                    let currency_label: String = self.fund_stand_currency.clone().map_or("None".to_string(), |currency| currency.to_string());
                    

                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::remainder().at_least(120.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                egui::Grid::new("fund_stand")
                                    .num_columns(3)
                                    .spacing([45.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Currency:").on_hover_text("Currency to which convert all amounts. Select None to avoid converting to a single currency.");
                                        ComboBox::from_id_salt("Fund stand currency")
                                            .selected_text(format!("{}", currency_label))
                                            .show_ui(ui, |ui| {
                                                for possible_fund_stand_currency in Currency::iter() {
                                                    ui.selectable_value(
                                                        &mut self.fund_stand_currency,
                                        Some(possible_fund_stand_currency.clone()),
                                        format!("{possible_fund_stand_currency}"),
                                        );
                                                }
                                                ui.selectable_value(
                                                    &mut self.fund_stand_currency,
                                                    None,
                                                    String::from("None")
                                                    );
                                            });
                                        ui.end_row();

                                        ui.label("");
                                        if ui.button("Generate!").clicked() {
                                            match self.database.current_fund_stand(
                                                self.fund_stand_currency.as_ref()
                                            ) {
                                                Ok(s) => {self.fund_stand_csv = s; self.fund_stand_csv_correct = true;},
                                                Err(e) => {self.fund_stand_csv_correct = false; self.throw_error(e);}
                                            }
                                        }

                                    });
                                ui.separator();
                            });
                            if self.fund_stand_csv_correct{
                            strip.cell(|ui| {
                                                TableBuilder::new(ui)
                                        .columns(Column::auto().resizable(true), column_count)
                                        .striped(true)
                                        .cell_layout(Layout::right_to_left(Align::Center))
                                        .header(20.0, |mut header| {
                                            for column_name in header_line.split(",") {
                                                header.col(|ui| {
                                                    ui.strong(column_name)
                                                        .on_hover_text(column_name);
                                                });
                                            }
                                        })
                                        .body(|mut body| {
                                            for row_line in row_lines {
                                                body.row(30.0, |mut row_ui| {
                                                    for element in row_line.split(",") {
                                                        row_ui.col(|ui| {
                                                                ui.label(element);
                                                        });
                                                    }
                                                });
                                            }                         
                                        });
                                ui.separator();
                            });}
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_fund_stand_window = false;
                }
            },
        )
    }
    pub fn handle_show_expenses_evolution_window (&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("expenses_evolution_summary_window"),
            egui::ViewportBuilder::default()
                .with_title("Expenses evolution summary window")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    let expenses_evolution_csv = self.expenses_evolution_csv.clone();
                    let header_line: String = expenses_evolution_csv.split("\n").collect::<Vec<&str>>()[0].to_string();
                    let row_lines: Vec<&str> = expenses_evolution_csv.split("\n").collect::<Vec<&str>>()[1..].to_vec();
                    let column_count: usize = header_line.split(",").count();
                    let currency_label: String = self.expenses_evolution_currency.to_string();
                    let time_unit_label: String = self.expenses_evolution_time_unit.to_string();

                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::initial(240.0))
                        .size(Size::remainder().at_least(10.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                egui::Grid::new("expenses_evolution")
                                    .num_columns(3)
                                    .spacing([45.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Currency:").on_hover_text("Currency on which to express expense ammounts.");
                                        ComboBox::from_id_salt("Expenses evolution currency")
                                            .selected_text(format!("{}", currency_label))
                                            .show_ui(ui, |ui| {
                                                for possible_expenses_evolution_currency in Currency::iter() {
                                                    ui.selectable_value(
                                                        &mut self.expenses_evolution_currency,
                                        possible_expenses_evolution_currency.clone(),
                                        format!("{possible_expenses_evolution_currency}"),
                                        );
                                                }
                                            });
                                        ui.end_row();

                                        ui.label("Time unit:").on_hover_text("Time unit to aggregate expenses.");
                                        ComboBox::from_id_salt("Expenses evolution time unit")
                                            .selected_text(format!("{}", time_unit_label))
                                            .show_ui(ui, |ui| {
                                                for possible_expenses_evolution_time_unit in TimeUnit::iter() {
                                                    ui.selectable_value(
                                                        &mut self.expenses_evolution_time_unit,
                                        possible_expenses_evolution_time_unit.clone(),
                                        format!("{possible_expenses_evolution_time_unit}"),
                                        );
                                                }
                                            });
                                        ui.end_row();

                                        ui.label("");
                                        if ui.button("Generate!").clicked() {
                                            match self.database.evolution_table(
                                                &self.expenses_evolution_currency,
                                                &self.expenses_evolution_time_unit,
                                            ) {
                                                Ok(s) => {self.expenses_evolution_csv = s; self.expenses_evolution_csv_correct = true;},
                                                Err(e) => {self.expenses_evolution_csv_correct = false; self.throw_error(e);}}
                                        }

                                    });
                                ui.separator();
                            });
                            if self.expenses_evolution_csv_correct {
                            strip.cell(|ui| {
                                TableBuilder::new(ui)
                                        .columns(Column::auto().resizable(true), column_count)
                                        .striped(true)
                                        .cell_layout(Layout::right_to_left(Align::Center))
                                        .header(20.0, |mut header| {
                                            for column_name in header_line.split(",") {
                                                header.col(|ui| {
                                                    ui.strong(column_name)
                                                        .on_hover_text(column_name);
                                                });
                                            }
                                        })
                                        .body(|mut body| {
                                            for row_line in row_lines {
                                                body.row(30.0, |mut row_ui| {
                                                    for element in row_line.split(",") {
                                                        row_ui.col(|ui| {
                                                                ui.label(element);
                                                        });
                                                    }
                                                });
                                            }                         
                                        });
                            });
                        }
                            strip.cell(|ui| {ui.separator();});
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_expenses_evolution_window = false;
                }
            },
        )
    }
}
