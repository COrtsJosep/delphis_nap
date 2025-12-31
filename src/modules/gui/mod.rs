pub mod browsing;
pub mod error;
pub mod inputting;
pub mod plotting;
pub mod summarizing;

use super::database::summaries::TimeUnit;
use crate::modules::database::plotter::BarplotType;
use crate::modules::database::*;
use crate::modules::financial::*;
use chrono::{Local, NaiveDate};
use derivative::*;
use eframe::egui;
use egui::PopupCloseBehavior;
use egui_extras::{Size, StripBuilder};

const WINDOW_HEIGHT: f32 = 400.0;
const WINDOW_WIDTH: f32 = 600.0;

#[derive(Derivative)]
#[derivative(Default)]
pub struct AppState {
    show_input_entity_window: bool,
    show_input_account_window: bool,
    show_input_party_window: bool,
    show_input_transaction_window: bool,
    show_expense_summary_window: bool,
    show_fund_stand_window: bool,
    show_browse_last_transactions_window: bool,
    show_browse_last_fund_movements_window: bool,
    show_fund_evolution_plot_window: bool,
    show_expense_category_plot_window: bool,
    show_expenses_evolution_window: bool,
    show_error_window: bool,

    error_message: String,

    database: DataBase,

    entity_name: String,
    entity_country: String,
    entity_type: EntityType,
    entity_subtype: String,

    account_name: String,
    account_country: String,
    account_currency: Currency,
    account_type: AccountType,
    account_initial_balance: f64,
    account_initial_balance_tentative: String,

    party: Party,

    transaction_value: f64,
    transaction_value_tentative: String,
    transaction_currency: Currency,
    #[derivative(Default(value = "Local::now().date_naive()"))]
    transaction_date: NaiveDate,
    transaction_category: String,
    transaction_subcategory: String,
    transaction_description: String,
    transaction_entity_id: i64,
    transaction_entity_string: String,
    transaction_account_id: i64,
    transaction_account_string: String,
    transaction_type: TransactionType,
    transaction_filter: String,
    #[derivative(Default(value = "PopupCloseBehavior::IgnoreClicks"))]
    transaction_entity_popup: PopupCloseBehavior,

    expense_summary_csv: String,
    expense_summary_csv_correct: bool,
    #[derivative(Default(value = "Local::now().date_naive()"))]
    expense_summary_date_from: NaiveDate,
    #[derivative(Default(value = "Local::now().date_naive()"))]
    expense_summary_date_to: NaiveDate,
    expense_summary_currency: Currency,

    fund_stand_csv: String,
    fund_stand_csv_correct: bool,
    fund_stand_currency: Option<Currency>,

    expenses_evolution_csv: String,
    expenses_evolution_csv_correct: bool,
    expenses_evolution_currency: Currency,
    expenses_evolution_time_unit: TimeUnit,

    last_transactions_csv: String,
    last_transactions_csv_correct: bool,
    last_transactions_n: usize,
    last_transactions_n_temptative: String,

    last_fund_movements_csv: String,
    last_fund_movements_csv_correct: bool,
    last_fund_movements_n: usize,
    last_fund_movements_n_temptative: String,

    #[derivative(Default(value = "-1"))]
    browse_account_id: i64,
    browse_account_string: String,

    fund_evolution_plot_currency: Currency,

    expense_category_plot_currency: Currency,
    expense_category_plot_type: BarplotType,
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> () {
        egui_extras::install_image_loaders(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(20.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        ui.vertical_centered_justified(|ui| {
                            if ui.button("Add transactions").clicked() {
                                self.show_input_party_window = true;
                                self.show_input_transaction_window = true;
                            };
                            ui.end_row();

                            ui.menu_button("Summaries", |ui| {
                                if ui.button("Expenses by Category").clicked() {
                                    self.show_expense_summary_window = true;
                                }
                                if ui.button("Funds by Account").clicked() {
                                    self.show_fund_stand_window = true;
                                }
                                if ui.button("Expenses Evolution").clicked() {
                                    self.show_expenses_evolution_window = true;
                                }
                            });
                            ui.end_row();

                            ui.menu_button("Plotting", |ui| {
                                if ui.button("Funds Evolution").clicked() {
                                    self.show_fund_evolution_plot_window = true;
                                }
                                if ui.button("Expenses by Category and Month").clicked() {
                                    self.show_expense_category_plot_window = true;
                                }
                            });
                            ui.end_row();

                            ui.menu_button("Browsing", |ui| {
                                if ui.button("Last transactions").clicked() {
                                    self.show_browse_last_transactions_window = true;
                                }
                                if ui.button("Last fund movements").clicked() {
                                    self.show_browse_last_fund_movements_window = true;
                                }
                            });
                            ui.end_row();
                        });
                    });
                });
        });

        if self.show_input_entity_window {
            self.handle_show_input_entity_window(ctx);
        }

        if self.show_input_account_window {
            self.handle_show_input_account_window(ctx);
        }

        if self.show_input_party_window {
            self.handle_show_input_party_window(ctx);
        }

        if self.show_input_transaction_window {
            self.handle_show_input_transaction_window(ctx)
        }

        if self.show_expense_summary_window {
            self.handle_show_expense_summary_window(ctx)
        }

        if self.show_fund_stand_window {
            self.handle_show_fund_stand_window(ctx)
        }

        if self.show_expenses_evolution_window {
            self.handle_show_expenses_evolution_window(ctx)
        }

        if self.show_browse_last_transactions_window {
            self.handle_show_browse_last_transactions_window(ctx)
        }

        if self.show_browse_last_fund_movements_window {
            self.handle_show_browse_last_fund_movements_window(ctx);
        }

        if self.show_fund_evolution_plot_window {
            self.handle_show_fund_evolution_plot(ctx);
        }

        if self.show_expense_category_plot_window {
            self.handle_show_expense_category_plot(ctx);
        }

        if self.show_error_window {
            self.handle_show_error_window(ctx);
        }
    }
}
