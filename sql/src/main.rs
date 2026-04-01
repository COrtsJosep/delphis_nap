pub mod financial_database;
pub mod table_records;
use crate::financial_database::FinancialDataBase;

#[tokio::main]
async fn main() {
    let financial_data_base = FinancialDataBase::init().await;
    match financial_data_base {
        Ok(_e) => (),
        Err(e) => println!("{:}", e),
    }
}
