use crate::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui::ComboBox;
use egui::{Align, Color32, Layout};
use egui_extras::*;
use egui_async::StateWithData;

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

                egui::CentralPanel::default().show_inside(ctx, |ui| {
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
                                            let last_transactions_n = self
                                                .last_transactions_n_temptative
                                                .parse::<i64>()
                                                .expect("Failed to parse the number of last transactions.");
                                            let db = self.financial_database.clone();
                                            let fut =
                                                async move { db.last_transactions(last_transactions_n).await };
                                            self.last_transaction_views_bind.request(fut);
                                        }
                                    }
                                });
                                ui.separator();
                            });
                            match self.last_transaction_views_bind.state() {
                                StateWithData::Finished(last_transaction_views) => {
                                    strip.cell(|ui| {
                                        TableBuilder::new(ui)
                                            .columns(Column::auto().resizable(true), 9)
                                            .striped(true)
                                            .cell_layout(Layout::right_to_left(Align::Center))
                                            .header(20.0, |mut header| {
                                                for column_name in ["Type", "Date", "Value", "Currency", "Account Name", "Category",  "Subcategory", "Description", ""] {
                                                    header.col(|ui| {
                                                        ui.strong(column_name).on_hover_text(column_name);
                                                    });
                                                }
                                            })
                                        .body(|mut body| {
                                            body.row(30.0, |mut row_ui| {
                                                for transaction_view in last_transaction_views.clone() {
                                                    row_ui.col(|ui| {ui.label(transaction_view.transaction_type.clone());});
                                                    row_ui.col(|ui| {ui.label(transaction_view.date.clone());});
                                                    row_ui.col(|ui| {ui.label(format!("{:.2}", transaction_view.value));});
                                                    row_ui.col(|ui| {ui.label(transaction_view.currency.clone());});
                                                    row_ui.col(|ui| {ui.label(transaction_view.name.clone());});
                                                    row_ui.col(|ui| {ui.label(transaction_view.category.clone());});
                                                    row_ui.col(|ui| {ui.label(transaction_view.subcategory.clone());});
                                                    row_ui.col(|ui| {ui.label(transaction_view.description.clone());});
                                                    row_ui.col(|ui| {
                                                        if ui
                                                            .button("Edit/Remove")
                                                            .on_hover_text("Removes the party from the database, and launches the input menu with an equal party already loaded")
                                                            .clicked() {
                                                            let db = self.financial_database.clone();
                                                            let party_id = transaction_view.party_id;
                                                            let fut = async move { db.party(party_id).await };
                                                            self.party_bind.request(fut);
                                                            match self.party_bind.state() {
                                                                StateWithData::Finished(party) => {
                                                                    let db = self.financial_database.clone();
                                                                    self.party = party.clone();
                                                                    let fut = async move { db.delete_party(party_id).await };
                                                                    self.delete_party_bind.request(fut);
                                                                    match self.delete_party_bind.state() {
                                                                        StateWithData::Finished(_) => {
                                                                            self.show_input_party_window = true;
                                                                            self.show_browse_last_transactions_window = false;
                                                                        },
                                                                        StateWithData::Failed(e) => {
                                                                            self.error_message = e.to_string();
                                                                            self.show_error_window = true;
                                                                        },
                                                                        _ => {},
                                                                    };
                                                                },
                                                                StateWithData::Failed(e) => {
                                                                    self.error_message = e.to_string();
                                                                    self.show_error_window = true;
                                                                },
                                                                _ => {},
                                                            };
                                                        }
                                                    });
                                                }
                                            });
                                        });
                                        ui.separator();
                                    });
                                },
                                StateWithData::Failed(e) => {
                                    self.error_message = e.to_string();
                                    self.show_error_window = true;
                                },
                                _ => {},
                            };
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_browse_last_transactions_window = false;
                }
            },
            );
    }

    pub fn handle_show_browse_last_fund_movements_window(&mut self, ctx: &egui::Context) -> () {
        self.browse_account_string = match self.browse_account_id >= 0 {
            true => {
                let browse_account_id = self.browse_account_id;
                let db = self.financial_database.clone();
                let fut = async move { db.account(browse_account_id).await };
                self.account_bind.request(fut);
                match self.account_bind.state() {
                    StateWithData::Finished(account) => account.to_string(),
                    StateWithData::Failed(e) => {
                        self.error_message = e.to_string();
                        self.show_error_window = true;
                        String::default()
                    },
                    _ => String::default(), 
                }
            }
            false => String::from("All accounts"),
        };
        
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

                egui::CentralPanel::default().show_inside(ctx, |ui| {
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
                                                let db = self.financial_database.clone();
                                                let fut = async move { db.iter_account_ids().await };
                                                self.account_ids_bind.request(fut);
                                                match self.account_ids_bind.state() {
                                                    StateWithData::Finished(iterator) => {
                                                        for account_id in iterator.clone() {
                                                            let db = self.financial_database.clone();
                                                            let fut = async move { db.account(account_id).await };
                                                            self.account_bind.request(fut);
                                                            match self.account_bind.state() {
                                                                StateWithData::Finished(account) => {
                                                                    ui.selectable_value(
                                                                        &mut self.browse_account_id,
                                                                        account_id,
                                                                        format!(
                                                                            "{:}",
                                                                            account.to_string()
                                                                        ),
                                                                    );
                                                                },
                                                                StateWithData::Failed(e) => {
                                                                    self.error_message = e.to_string();
                                                                    self.show_error_window = true;
                                                                },
                                                                _ => {},
                                                            };
                                                        }
                                                    },
                                                    StateWithData::Failed(e) => {
                                                        self.error_message = e.to_string();
                                                        self.show_error_window = true;
                                                    },
                                                    _ => {},
                                                };
                                            });
                                        ui.label("");
                                        ui.end_row();
                                    });
                                ui.separator();
                                ui.vertical_centered_justified(|ui| {
                                    if self.is_valid_last_fund_movements_n() {
                                        if ui.button("Generate!").clicked() {
                                            let last_fund_movements_n = self
                                                .last_fund_movements_n_temptative
                                                .parse::<i64>()
                                                .expect("Failed to parse the number of last fund_movements.");
                                            let db = self.financial_database.clone();
                                            let browse_account_id = self.browse_account_id;
                                            let fut = async move { db.last_fund_movements(last_fund_movements_n, browse_account_id).await };
                                            self.last_fund_movements_bind.request(fut);
                                        }
                                    }
                                });
                                ui.separator();
                            });
                            match self.last_fund_movements_bind.state() {
                                StateWithData::Finished(last_fund_movement_views) => {
                                    strip.cell(|ui| {
                                        TableBuilder::new(ui)
                                            .columns(Column::auto().resizable(true), 6)
                                            .striped(true)
                                            .cell_layout(Layout::right_to_left(Align::Center))
                                            .header(20.0, |mut header| {
                                                for column_name in ["Type", "Date", "Value", "Currency", "Account Name", ""] {
                                                    header.col(|ui| {
                                                        ui.strong(column_name).on_hover_text(column_name);
                                                    });
                                                }
                                            })
                                        .body(|mut body| {
                                            for fund_movement_view in last_fund_movement_views {
                                                body.row(30.0, |mut row_ui| {
                                                    row_ui.col(|cell_ui| {cell_ui.label(fund_movement_view.fund_movement_type.clone());});
                                                    row_ui.col(|cell_ui| {cell_ui.label(fund_movement_view.date.clone());});
                                                    row_ui.col(|cell_ui| {cell_ui.label(format!("{:.2}", fund_movement_view.value));});
                                                    row_ui.col(|cell_ui| {cell_ui.label(fund_movement_view.currency.clone());});
                                                    row_ui.col(|cell_ui| {cell_ui.label(fund_movement_view.name.clone());});
                                                    row_ui.col(|cell_ui| {
                                                        if cell_ui
                                                            .button("Edit/Remove")
                                                            .on_hover_text("Removes the party from the database, and launches the input menu with an equal party already loaded")
                                                            .clicked() {
                                                            let db = self.financial_database.clone();
                                                            let party_id = fund_movement_view.party_id;
                                                            let fut = async move { db.party(party_id).await };
                                                            self.party_bind.request(fut);
                                                            match self.party_bind.state() {
                                                                StateWithData::Finished(party) => {
                                                                    let db = self.financial_database.clone();
                                                                    self.party = party.clone();
                                                                    let fut = async move { db.delete_party(party_id).await };
                                                                    self.delete_party_bind.request(fut);
                                                                    match self.delete_party_bind.state() {
                                                                        StateWithData::Finished(_) => {
                                                                            self.show_input_party_window = true;
                                                                            self.show_browse_last_fund_movements_window = false;
                                                                        },
                                                                        StateWithData::Failed(e) => {
                                                                            self.error_message = e.to_string();
                                                                            self.show_error_window = true;
                                                                        },
                                                                        _ => {},
                                                                    };
                                                                },
                                                                StateWithData::Failed(e) => {
                                                                    self.error_message = e.to_string();
                                                                    self.show_error_window = true;
                                                                },
                                                                _ => {},
                                                            };
                                                        }
                                                    });
                                                });
                                            }
                                        });
                                        ui.separator();
                                    });
                                },
                                StateWithData::Failed(e) => {
                                    self.error_message = e.to_string();
                                    self.show_error_window = true;
                                },
                                _ => {},
                            };
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_browse_last_fund_movements_window = false;
                }
            },
            );
    }
}
