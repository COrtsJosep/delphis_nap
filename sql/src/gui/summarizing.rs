use crate::financial::*;
use crate::financial_database::summaries::TimeUnit;
use crate::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use egui::{Align, ComboBox, Layout};
use egui_extras::*;
use sqlx::Row;
use strum::IntoEnumIterator;

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
                                        async {
                                        if ui.button("Generate!").clicked() {
                                            match self.financial_database.expenses_summary(
                                                self.expense_summary_date_from,
                                                self.expense_summary_date_to,
                                                &self.expense_summary_currency
                                            ).await {
                                                Ok(v) => {self.expense_summary_rows = v;},
                                                Err(e) => {self.throw_sqlx_error(e);}}
                                        }};
                                    });
                                ui.separator();
                            });
                            if !self.expense_summary_rows.is_empty() {
                            strip.cell(|ui| {
                                TableBuilder::new(ui)
                                        .columns(Column::auto().resizable(true), 6)
                                        .striped(true)
                                        .cell_layout(Layout::right_to_left(Align::Center))
                                        .header(20.0, |mut header| {
                                            for column_name in vec!["Category", "Subcategory", "Value", "Value per Day", "% of Total Expenses", "% of Total Income"] {
                                                header.col(|ui| {
                                                    ui.strong(column_name)
                                                        .on_hover_text(column_name);
                                                });
                                            }
                                        })
                                        .body(|mut body| {
                                            for expense_summary_row in &self.expense_summary_rows {
                                                body.row(30.0, |mut row_ui| {
                                                    if expense_summary_row.category == "Total".to_string() {
                                                        row_ui.col(|ui| {ui.strong(expense_summary_row.category.clone());});
                                                        row_ui.col(|ui| {ui.strong(expense_summary_row.subcategory.clone());});
                                                        row_ui.col(|ui| {ui.strong(format!("{:.2}", expense_summary_row.value));});
                                                        row_ui.col(|ui| {ui.strong(format!("{:.2}", expense_summary_row.value_day));});
                                                        row_ui.col(|ui| {ui.strong(format!("{:.2}%", 100.0 * expense_summary_row.value_total_expenses));});
                                                        row_ui.col(|ui| {ui.strong(format!("{:.2}%", 100.0 * expense_summary_row.value_total_incomes));});
                                                    } else {
                                                        row_ui.col(|ui| {ui.label(expense_summary_row.category.clone());});
                                                        row_ui.col(|ui| {ui.label(expense_summary_row.subcategory.clone());});
                                                        row_ui.col(|ui| {ui.label(format!("{:.2}", expense_summary_row.value));});
                                                        row_ui.col(|ui| {ui.label(format!("{:.2}", expense_summary_row.value_day));});
                                                        row_ui.col(|ui| {ui.label(format!("{:.2}%", 100.0 * expense_summary_row.value_total_expenses));});
                                                        row_ui.col(|ui| {ui.label(format!("{:.2}%", 100.0 * expense_summary_row.value_total_incomes));});
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
                    let currency_label: String = self.current_fund_stand_currency.clone().map_or("None".to_string(), |currency| currency.to_string());


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
                                                for possible_current_fund_stand_currency in Currency::iter() {
                                                    ui.selectable_value(
                                                        &mut self.current_fund_stand_currency,
                                        Some(possible_current_fund_stand_currency.clone()),
                                        format!("{possible_current_fund_stand_currency}"),
                                        );
                                                }
                                                ui.selectable_value(
                                                    &mut self.current_fund_stand_currency,
                                                    None,
                                                    String::from("None")
                                                    );
                                            });
                                        ui.end_row();

                                        ui.label("");
                                        async {
                                        if ui.button("Generate!").clicked() {
                                            match self.financial_database.current_fund_stand(
                                                self.current_fund_stand_currency.as_ref()
                                            ).await {
                                                Ok(v) => {self.current_fund_stand_rows = v;},
                                                Err(e) => {self.throw_sqlx_error(e);}
                                            }
                                        }
                                        };

                                    });
                                ui.separator();
                            });
                            if self.fund_stand_csv_correct{
                            strip.cell(|ui| {
                                TableBuilder::new(ui)
                                        .columns(Column::auto().resizable(true), match self.current_fund_stand_currency.clone() {Some(_c) => 4, None => 5})
                                        .striped(true)
                                        .cell_layout(Layout::right_to_left(Align::Center))
                                        .header(20.0, |mut header| {
                                            let column_names: Vec<String> = match self.current_fund_stand_currency.clone() {
                                                Some(c) => vec!["Name".into(), "Country".into(), "Account Type".into(), c.to_string()],
                                                None => vec!["Name".into(), "Country".into(), "Currency".into(), "Account Type".into(), "Value".into()]
                                            };
                                            for column_name in column_names {
                                                header.col(|ui| {
                                                    ui.strong(column_name.clone())
                                                        .on_hover_text(column_name);
                                                });
                                            }
                                        })
                                        .body(|mut body| {
                                            for current_fund_stand_row in &self.current_fund_stand_rows {
                                                body.row(30.0, |mut row_ui| {
                                                    row_ui.col(|ui| {
                                                    ui.label(current_fund_stand_row.name.clone());});
                                                    row_ui.col(|ui| {
                                                    ui.label(current_fund_stand_row.country.clone());});
                                                    match self.current_fund_stand_currency.clone() {
                                                        Some(_c) => {},
                                                        None => {row_ui.col(|ui| {
ui.label(current_fund_stand_row.currency.clone());});}
                                                    }
                                                    row_ui.col(|ui| {
ui.label(current_fund_stand_row.account_type.clone());});
                                                    row_ui.col(|ui| {
ui.label(format!("{:.2}", current_fund_stand_row.current_value));});
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
    pub fn handle_show_expenses_evolution_window(&mut self, ctx: &egui::Context) -> () {
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
                                        ui.label("Currency:").on_hover_text(
                                            "Currency on which to express expense ammounts.",
                                        );
                                        ComboBox::from_id_salt("Expenses evolution currency")
                                            .selected_text(format!("{}", currency_label))
                                            .show_ui(ui, |ui| {
                                                for possible_expenses_evolution_currency in
                                                    Currency::iter()
                                                {
                                                    ui.selectable_value(
                                                        &mut self.expenses_evolution_currency,
                                        possible_expenses_evolution_currency.clone(),
                                        format!("{possible_expenses_evolution_currency}"),
                                        );
                                                }
                                            });
                                        ui.end_row();

                                        ui.label("Time unit:")
                                            .on_hover_text("Time unit to aggregate expenses.");
                                        ComboBox::from_id_salt("Expenses evolution time unit")
                                            .selected_text(format!("{}", time_unit_label))
                                            .show_ui(ui, |ui| {
                                                for possible_expenses_evolution_time_unit in
                                                    TimeUnit::iter()
                                                {
                                                    ui.selectable_value(
                                                        &mut self.expenses_evolution_time_unit,
                                        possible_expenses_evolution_time_unit.clone(),
                                        format!("{possible_expenses_evolution_time_unit}"),
                                        );
                                                }
                                            });
                                        ui.end_row();

                                        ui.label("");
                                        async {
                                            if ui.button("Generate!").clicked() {
                                                match self
                                                    .financial_database
                                                    .evolution_table(
                                                        &self.expenses_evolution_currency,
                                                        &self.expenses_evolution_time_unit,
                                                    )
                                                    .await
                                                {
                                                    Ok(v) => {
                                                        self.expenses_evolution_unique_categories =
                                                            v.0;
                                                        self.expenses_evolution_rows = v.1;
                                                    }
                                                    Err(e) => {
                                                        self.throw_sqlx_error(e);
                                                    }
                                                }
                                            }
                                        };
                                    });
                                ui.separator();
                            });
                            if self.expenses_evolution_csv_correct {
                                strip.cell(|ui| {
                                    TableBuilder::new(ui)
                                        .columns(
                                            Column::auto().resizable(true),
                                            self.expenses_evolution_unique_categories.len() + 1,
                                        )
                                        .striped(true)
                                        .cell_layout(Layout::right_to_left(Align::Center))
                                        .header(20.0, |mut header| {
                                            header.col(|ui| {
                                                ui.strong(
                                                    self.expenses_evolution_time_unit.to_string(),
                                                )
                                                .on_hover_text(
                                                    self.expenses_evolution_time_unit.to_string(),
                                                );
                                            });
                                            for column_name in
                                                &self.expenses_evolution_unique_categories
                                            {
                                                header.col(|ui| {
                                                    ui.strong(column_name)
                                                        .on_hover_text(column_name);
                                                });
                                            }
                                        })
                                        .body(|mut body| {
                                            for expenses_evolution_row in
                                                &self.expenses_evolution_rows
                                            {
                                                body.row(30.0, |mut row_ui| {
                                                    row_ui.col(|ui| {
                                                        ui.label(
                                                            expenses_evolution_row
                                                                .get::<String, usize>(0),
                                                        );
                                                    });
                                                    for i in 1..expenses_evolution_row.len() {
                                                        row_ui.col(|ui| {
                                                            ui.label(format!(
                                                                "{:.2}",
                                                                expenses_evolution_row
                                                                    .get::<f64, usize>(i)
                                                            ));
                                                        });
                                                    }
                                                });
                                            }
                                        });
                                });
                            }
                            strip.cell(|ui| {
                                ui.separator();
                            });
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_expenses_evolution_window = false;
                }
            },
        )
    }
}
