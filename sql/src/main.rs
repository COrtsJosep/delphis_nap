pub mod financial;
pub mod financial_database;
pub mod gui;
pub mod table_records;

use crate::financial_database::FinancialDataBase;
use crate::gui::AppState;
use eframe::egui;

const FINANCIAL_DATABASE_URL: &str = "sqlite://./data/financial_database.sqlite";
const TEST_FINANCIAL_DATABASE_URL: &str = "sqlite://./data_fake/financial_database.sqlite";
const TEST_ORIGINAL_FINANCIAL_DATABASE_URL: &str =
    "sqlite://./data_fake/financial_database_original.sqlite";

#[tokio::main]
async fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([360.0, 100.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Financial Application",
        options,
        Box::new(|_cc| Ok(Box::<AppState>::default())),
    )
}
