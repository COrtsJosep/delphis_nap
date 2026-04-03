pub mod financial;
pub mod financial_database;
pub mod table_records;
use crate::financial_database::FinancialDataBase;
use chrono::{Datelike, NaiveDate};
use std::io::Cursor;

#[derive(Debug, serde::Deserialize)]
struct ECBRecord {
    #[serde(rename = "TIME_PERIOD")]
    date: String,
    #[serde(rename = "OBS_VALUE")]
    value: f64,
}

#[tokio::main]
async fn main() {
    let response = reqwest::get("https://data-api.ecb.europa.eu/service/data/EXR/D.CHF.EUR.SP00.A?format=csvdata&detail=dataonly&startPeriod=2026-01-01").await.unwrap();
    let csv_data = response.bytes().await.unwrap();

    let cursor = Cursor::new(csv_data);

    let mut reader = csv::Reader::from_reader(cursor);
    for record in reader.deserialize() {
        let ecb_record: ECBRecord = record.unwrap();
        let date = NaiveDate::parse_from_str(ecb_record.date.as_str(), "%Y-%m-%d").unwrap();
        println!("{:?}", date.weekday());
        println!("{:?}", ecb_record);
    }

    //let financial_data_base = FinancialDataBase::init().await;
    //match financial_data_base {
    //    Ok(_e) => (),
    //    Err(e) => println!("{:}", e),
    //}
}
