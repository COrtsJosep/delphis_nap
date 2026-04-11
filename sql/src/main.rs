pub mod financial;
pub mod financial_database;
pub mod table_records;
use crate::financial_database::FinancialDataBase;

const FINANCIAL_DATABASE_URL: &str = "sqlite://./data/financial_database.sqlite";
const TEST_FINANCIAL_DATABASE_URL: &str = "sqlite://./data_fake/financial_database.sqlite";
const TEST_ORIGINAL_FINANCIAL_DATABASE_URL: &str =
    "sqlite://./data_fake/financial_database_original.sqlite";

#[tokio::main]
async fn main() {
    let financial_data_base = FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL).await;
    match financial_data_base {
        Ok(_e) => (),
        Err(e) => println!("{:}", e),
    }
}
