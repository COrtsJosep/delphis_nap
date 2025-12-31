use crate::modules::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui::ComboBox;
use egui::{Align, Color32, Layout};
use egui_extras::*;

impl AppState {
    fn is_valid_last_transactions_n(&self) -> bool {
        let parsing_result = self.last_transactions_n_temptative.parse::<usize>();
        match parsing_result {
            Ok(_value) => true,
            Err(_e) => false,
        }
    }

    fn is_valid_last_fund_movements_n(&self) -> bool {
        let parsing_result = self.last_fund_movements_n_temptative.parse::<usize>();
        match parsing_result {
            Ok(_value) => true,
            Err(_e) => false,
        }
    }

    pub fn handle_show_browse_last_transactions_window(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("browse_last_transactions_window"),
            egui::ViewportBuilder::default()
            .with_title("Last transactions window")
            .with_inner_size([WINDOW_WIDTH * 1.2, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    let last_transactions_csv = self.last_transactions_csv.clone();
                    let header_line: String =
                        last_transactions_csv.split("\n").collect::<Vec<&str>>()[0].to_string();
                    let row_lines: Vec<&str> =
                        last_transactions_csv.split("\n").collect::<Vec<&str>>()[1..].to_vec();
                    let column_count: usize = header_line.split(",").count();

                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::remainder().at_least(120.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                egui::Grid::new("last_transactions")
                                    .num_columns(3)
                                    .spacing([45.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Number of records:").on_hover_text("Number of income/expense records to show.");
                                        ui.text_edit_singleline(&mut self.last_transactions_n_temptative);
                                        if self.is_valid_last_transactions_n() {
                                            ui.colored_label(
                                                Color32::from_rgb(110, 255, 110),
                                                "Valid number of records!",
                                            );

                                        } else {
                                            ui.colored_label(
                                                Color32::from_rgb(255, 0, 0),
                                                "Invalid number of records!",
                                            );
                                        }
                                        ui.end_row();
                                    });
                                ui.separator();
                                ui.vertical_centered_justified(|ui| {
                                    if self.is_valid_last_transactions_n() {
                                        if ui.button("Generate!").clicked() {
                                            self.last_transactions_n = self.last_transactions_n_temptative.parse::<usize>().expect("Failed to parse the number of last transactions.");
                                            match self.database.last_transactions(self.last_transactions_n) {
                                                Ok(s) => {
                                                    self.last_transactions_csv = s; 
                                                    self.last_transactions_csv_correct = true;},
                                                Err(e) => {
                                                    self.last_transactions_csv_correct = false; 
                                                    self.throw_error(e);},
                                            }
                                        }
                                    }
                                });
                                ui.separator();
                            });
                            if self.last_transactions_csv_correct {
                            strip.cell(|ui| {
                                TableBuilder::new(ui)
                                    .columns(Column::auto().resizable(true), column_count)
                                    .striped(true)
                                    .cell_layout(Layout::right_to_left(Align::Center))
                                    .header(20.0, |mut header| {
                                        for column_name in header_line.split(",") {
                                            header.col(|ui| {
                                                ui.strong(column_name).on_hover_text(column_name);
                                            });
                                        }
                                    })
                                .body(|mut body| {
                                    for row_line in row_lines {
                                        body.row(30.0, |mut row_ui| {
                                            let mut i: usize = 0;
                                            for element in row_line.split(",") {
                                                row_ui.col(|ui| {
                                                    if i == column_count - 1 {
                                                        // index of the last column
                                                        if ui.button("Edit/Remove").on_hover_text("Removes the party from the database, and launches the input menu with an equal party already loaded").clicked() {
                                                            let party_id: i64 =
                                                                element.parse().unwrap();
                                                            match self.database.party(party_id) {
                                                                Ok(party) => { 
                                                                    self.party = party;
                                                                    self.database.delete_party(party_id);
                                                                    self.database.save();

                                                                    self.show_input_party_window = true;
                                                                    self.show_browse_last_transactions_window = false;
                                                                },
                                                                Err(e) => {self.throw_error(e);}
                                                            }
                                                        }
                                                    } else {
                                                        ui.label(element);
                                                    }
                                                });

                                                i += 1;
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
                    self.show_browse_last_transactions_window = false;
                }
            },
            );
    }

    pub fn handle_show_browse_last_fund_movements_window(&mut self, ctx: &egui::Context) -> () {
        if self.browse_account_id >= 0 {
            self.browse_account_string = self
                .database
                .account(self.browse_account_id)
                .unwrap() // safe due to how it is set
                .to_string();
            } else {
                self.browse_account_string = String::from("All accounts");
        }

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("browse_last_fund_movements_window"),
            egui::ViewportBuilder::default()
            .with_title("Last fund movements window")
            .with_inner_size([WINDOW_WIDTH * 1.2, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    let last_fund_movements_csv = self.last_fund_movements_csv.clone();
                    let header_line: String =
                        last_fund_movements_csv.split("\n").collect::<Vec<&str>>()[0].to_string();
                    let row_lines: Vec<&str> =
                        last_fund_movements_csv.split("\n").collect::<Vec<&str>>()[1..].to_vec();
                    let column_count: usize = header_line.split(",").count();

                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::remainder().at_least(120.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                egui::Grid::new("last_transactions")
                                    .num_columns(3)
                                    .spacing([45.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Number of records:").on_hover_text("Number of fund movement records to show.");
                                        ui.text_edit_singleline(&mut self.last_fund_movements_n_temptative);
                                        if self.is_valid_last_fund_movements_n() {
                                            ui.colored_label(
                                                Color32::from_rgb(110, 255, 110),
                                                "Valid number of records!",
                                            );
                                        } else {
                                            ui.colored_label(
                                                Color32::from_rgb(255, 0, 0),
                                                "Invalid number of records!",
                                            );
                                        }
                                        ui.end_row();

                                        ui.label("Account:")
                                            .on_hover_text("Account to filter for.");
                                        ComboBox::from_id_salt("Account")
                                            .selected_text(format!("{}", self.browse_account_string))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.browse_account_id, 
                                                    -1, 
                                                    String::from("All accounts")
                                                );
                                                match self.database.iter_account_ids() {
                                                    Ok(iterator) => {
                                                for account_id in iterator {
                                                    ui.selectable_value(
                                                        &mut self.browse_account_id,
                                                        account_id,
                                                        format!(
                                                            "{:}",
                                                            self.database
                                                            .account(account_id)
                                                            .unwrap() // safe because we iterate
                                                                      // over the ids!
                                                            .to_string()
                                                        ),
                                                    );

                                                }}, Err(e) => {self.throw_polars_error(e);}}
                                            });
                                        ui.label("");
                                        ui.end_row();
                                    });
                                ui.separator();
                                ui.vertical_centered_justified(|ui| {
                                    if self.is_valid_last_fund_movements_n() {
                                        if ui.button("Generate!").clicked() {
                                            self.last_fund_movements_n = self
                                                .last_fund_movements_n_temptative
                                                .parse::<usize>()
                                                .expect("Failed to parse the number of last fund_movements.");
                                            match self.database.last_fund_movements(self.last_fund_movements_n, self.browse_account_id) {
                                                Ok(s) => {self.last_fund_movements_csv = s; self.last_fund_movements_csv_correct = true;},
                                                Err(e) => {self.last_fund_movements_csv_correct = false; self.throw_error(e);},
                                            }
                                        }
                                    }
                                });
                                ui.separator();
                            });
                            if self.last_fund_movements_csv_correct {
                            strip.cell(|ui| {
                                TableBuilder::new(ui)
                                    .columns(Column::auto().resizable(true), column_count)
                                    .striped(true)
                                    .cell_layout(Layout::right_to_left(Align::Center))
                                    .header(20.0, |mut header| {
                                        for column_name in header_line.split(",") {
                                            header.col(|ui| {
                                                ui.strong(column_name).on_hover_text(column_name);
                                            });
                                        }
                                    })
                                .body(|mut body| {
                                    for row_line in row_lines {
                                        body.row(30.0, |mut row_ui| {
                                            let mut i = 0;
                                            for element in row_line.split(",") {
                                                row_ui.col(|ui| {
                                                    if i == column_count - 1 {
                                                        // index of the last column
                                                        if ui.button("Edit/Remove").on_hover_text("Removes the party from the database, and launches the input menu with an equal party already loaded").clicked() {
                                                            let party_id: i64 =
                                                                element.parse().unwrap();
                                                            match self.database.party(party_id) {
                                                                Ok(party) => {
                                                            self.party = party;
                                                            self.database.delete_party(party_id);
                                                            self.database.save();

                                                            self.show_input_party_window = true;
                                                            self.show_browse_last_fund_movements_window = false;},
                                                            Err(e) => {self.throw_error(e);}
                                                        }
                                                        }
                                                    } else {
                                                        ui.label(element);
                                                    }
                                                });

                                                i += 1;
                                            }
                                        });
                                    }
                                });
                                ui.separator();
                            });}
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_browse_last_fund_movements_window = false;
                }
            },
            );
    }
}
