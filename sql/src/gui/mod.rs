pub mod browsing;
pub mod error;
pub mod inputting;
pub mod plotting;
pub mod summarizing;

use crate::financial::*;
use crate::financial_database::{
    plotter::BarplotType, summaries::CurrentFundStandRow, summaries::ExpenseSummaryRow,
    summaries::TimeUnit, views::FundMovementView, views::TransactionView, FinancialDataBase,
};
use derivative::*;
use eframe::{egui, App, Frame};
use egui::Ui;
use egui_async;
use egui_async::Bind;
use egui_extras::{Size, StripBuilder};
use jiff::{civil::Date, Zoned};
use sqlx::sqlite::SqliteRow;
use std::vec::IntoIter;

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

    financial_database: FinancialDataBase,

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
    #[derivative(Default(value = "Bind::new(true)"))]
    party_bind: Bind<Party, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    delete_party_bind: Bind<(), sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    insert_party_bind: Bind<(), sqlx::Error>,
    
    #[derivative(Default(value = "Bind::new(true)"))]
    account_bind: Bind<Account, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    account_ids_bind: Bind<IntoIter<i64>, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    account_countries_bind: Bind<Vec<String>, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    insert_account_bind: Bind<i64, sqlx::Error>,
    
    #[derivative(Default(value = "Bind::new(true)"))]
    entity_bind: Bind<Entity, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    entity_ids_bind: Bind<IntoIter<i64>, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    entity_countries_bind: Bind<Vec<String>, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    entity_subtypes_bind: Bind<Vec<String>, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    insert_entity_bind: Bind<i64, sqlx::Error>,
    
    #[derivative(Default(value = "Bind::new(true)"))]
    transaction_categories_bind: Bind<Vec<String>, sqlx::Error>,
    #[derivative(Default(value = "Bind::new(true)"))]
    transaction_subcategories_bind: Bind<Vec<String>, sqlx::Error>,

    transaction_value: f64,
    transaction_value_tentative: String,
    transaction_currency: Currency,
    #[derivative(Default(value = "Zoned::now().date()"))]
    transaction_date: Date,
    transaction_category: String,
    transaction_subcategory: String,
    transaction_description: String,
    transaction_entity_id: i64,
    transaction_entity_string: String,
    transaction_account_id: i64,
    transaction_account_string: String,
    transaction_type: TransactionType,
    transaction_filter: String,

    #[derivative(Default(value = "Zoned::now().date()"))]
    expense_summary_date_from: Date,
    #[derivative(Default(value = "Zoned::now().date()"))]
    expense_summary_date_to: Date,
    expense_summary_currency: Currency,
    #[derivative(Default(value = "Bind::new(true)"))]
    expense_summary_bind: Bind<Vec<ExpenseSummaryRow>, sqlx::Error>,

    current_fund_stand_prospective_currency: Option<Currency>,
    current_fund_stand_current_currency: Option<Currency>,
    #[derivative(Default(value = "Bind::new(true)"))]
    current_fund_stand_bind: Bind<Vec<CurrentFundStandRow>, sqlx::Error>,

    expenses_evolution_currency: Currency,
    expenses_evolution_time_unit: TimeUnit,
    #[derivative(Default(value = "Bind::new(true)"))]
    expenses_evolution_table_bind: Bind<(Vec<String>, Vec<SqliteRow>), sqlx::Error>,

    last_transactions_n_temptative: String,
    #[derivative(Default(value = "Bind::new(true)"))]
    last_transaction_views_bind: Bind<Vec<TransactionView>, sqlx::Error>,

    last_fund_movements_n_temptative: String,
    #[derivative(Default(value = "Bind::new(true)"))]
    last_fund_movements_bind: Bind<Vec<FundMovementView>, sqlx::Error>,

    #[derivative(Default(value = "-1"))]
    browse_account_id: i64,
    browse_account_string: String,

    fund_evolution_plot_currency: Currency,
    #[derivative(Default(value = "Bind::new(true)"))]
    fund_evolution_plot_bind: Bind<(), sqlx::Error>,
    fund_evolution_plot_clear_requested: bool,

    expense_category_plot_currency: Currency,
    expense_category_plot_type: BarplotType,
    #[derivative(Default(value = "Bind::new(true)"))]
    expense_category_plot_bind: Bind<(), sqlx::Error>,
    expense_category_plot_clear_requested: bool,
}

impl App for AppState {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.plugin_or_default::<egui_async::EguiAsyncPlugin>();
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) -> () {
        egui::CentralPanel::default().show_inside(ui, |ui| {
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
            self.handle_show_input_entity_window(ui);
        }

        if self.show_input_account_window {
            self.handle_show_input_account_window(ui);
        }

        if self.show_input_party_window {
            self.handle_show_input_party_window(ui);
        }

        if self.show_input_transaction_window {
            self.handle_show_input_transaction_window(ui);
        }

        if self.show_expense_summary_window {
            self.handle_show_expense_summary_window(ui)
        }

        if self.show_fund_stand_window {
            self.handle_show_fund_stand_window(ui)
        }

        if self.show_expenses_evolution_window {
            self.handle_show_expenses_evolution_window(ui)
        }

        if self.show_browse_last_transactions_window {
            self.handle_show_browse_last_transactions_window(ui)
        }

        if self.show_browse_last_fund_movements_window {
            self.handle_show_browse_last_fund_movements_window(ui);
        }

        if self.show_fund_evolution_plot_window {
            self.handle_show_fund_evolution_plot(ui);
        }

        if self.show_expense_category_plot_window {
            self.handle_show_expense_category_plot(ui);
        }

        if self.show_error_window {
            self.handle_show_error_window(ui);
        }
    }
}
