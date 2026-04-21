mod palettes;
pub mod plotter;
pub mod ser_deser;
pub mod summaries;
pub mod views;

use crate::financial::Currency;
use crate::table_records::*;
use crate::FINANCIAL_DATABASE_URL;
use jiff::{civil::Date, civil::Weekday, Zoned};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};
use std::io::Cursor;
use std::path::Path;
use strum::IntoEnumIterator;
use tokio::runtime::Runtime;

const BASE_CURRENCY: Currency = Currency::EUR;
const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Clone)]
pub struct FinancialDataBase {
    pool: SqlitePool,
}

#[derive(Debug, serde::Deserialize)]
struct ECBRecord {
    #[serde(rename = "TIME_PERIOD")]
    date: String,
    #[serde(rename = "OBS_VALUE")]
    value: Option<f64>,
}

impl FinancialDataBase {
    async fn first_boot(financial_database_url: &str) -> Result<SqlitePool, sqlx::Error> {
        Sqlite::create_database(financial_database_url).await.expect("Attempted to create new SQLite database, but there already exists one! This should never happen.");
        println!("New database created!");
        let pool = SqlitePool::connect(financial_database_url).await?;
        let mut transaction = pool.begin().await?;

        // initalization of all six tables
        sqlx::query_file!("src/queries/table_creation/create_account_table.sql")
            .execute(&mut *transaction)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_entity_table.sql")
            .execute(&mut *transaction)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_expense_table.sql")
            .execute(&mut *transaction)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_fund_movement_table.sql")
            .execute(&mut *transaction)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_income_table.sql")
            .execute(&mut *transaction)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_party_table.sql")
            .execute(&mut *transaction)
            .await?;

        // now populate the empty tables
        let account_table_path = Path::new("data/account_table.csv");
        if account_table_path.exists() {
            let mut reader = csv::Reader::from_path(account_table_path).unwrap();
            for result in reader.deserialize() {
                let account_record: AccountRecord = result.unwrap();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_accounts.sql",
                    account_record.account_id,
                    account_record.name,
                    account_record.country,
                    account_record.currency,
                    account_record.account_type,
                    account_record.initial_balance,
                    account_record.creation_date,
                )
                .execute(&mut *transaction)
                .await?;
            }
        } else {
            // TODO: add default account
        }

        let entity_table_path = Path::new("data/entity_table.csv");
        if entity_table_path.exists() {
            let mut reader = csv::Reader::from_path(entity_table_path).unwrap();
            for result in reader.deserialize() {
                let entity_record: EntityRecord = result.unwrap();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_entities.sql",
                    entity_record.entity_id,
                    entity_record.name,
                    entity_record.country,
                    entity_record.entity_type,
                    entity_record.entity_subtype,
                    entity_record.creation_date,
                )
                .execute(&mut *transaction)
                .await?;
            }
        } else {
            // TODO: add default entity
        }

        let party_table_path = Path::new("data/party_table.csv");
        if party_table_path.exists() {
            let mut reader = csv::Reader::from_path(party_table_path).unwrap();
            for result in reader.deserialize() {
                let record: PartyRecord = result.unwrap();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_parties.sql",
                    record.party_id,
                    record.creation_date,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }

        let expense_table_path = Path::new("data/expense_table.csv");
        if expense_table_path.exists() {
            let mut reader = csv::Reader::from_path(expense_table_path).unwrap();
            for result in reader.deserialize() {
                let expense_record: ExpenseRecord = result.unwrap();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_expenses.sql",
                    expense_record.expense_id,
                    expense_record.value,
                    expense_record.currency,
                    expense_record.date,
                    expense_record.category,
                    expense_record.subcategory,
                    expense_record.description,
                    expense_record.entity_id,
                    expense_record.party_id,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }

        let fund_movement_table_path = Path::new("data/fund_movement_table.csv");
        if fund_movement_table_path.exists() {
            let mut reader = csv::Reader::from_path(fund_movement_table_path).unwrap();
            for result in reader.deserialize() {
                let record: FundMovementRecord = result.unwrap();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_fund_movements.sql",
                    record.fund_movement_id,
                    record.fund_movement_type,
                    record.value,
                    record.currency,
                    record.date,
                    record.account_id,
                    record.party_id,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }

        let income_table_path = Path::new("data/income_table.csv");
        if income_table_path.exists() {
            let mut reader = csv::Reader::from_path(income_table_path).unwrap();
            for result in reader.deserialize() {
                let record: IncomeRecord = result.unwrap();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_incomes.sql",
                    record.income_id,
                    record.value,
                    record.currency,
                    record.date,
                    record.category,
                    record.subcategory,
                    record.description,
                    record.entity_id,
                    record.party_id,
                )
                .execute(&mut *transaction)
                .await?;
            }
        }
        transaction.commit().await?;

        Ok(pool)
    }

    async fn init_currency_exchange(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query_file!("src/queries/table_creation/create_currency_exchange_table.sql")
            .execute(pool)
            .await?;

        let last_date: String =
            match sqlx::query!("select max(date) as max_date from currency_exchanges")
                .fetch_one(pool)
                .await
            {
                Ok(row) => match row.max_date {
                    Some(date) => date,
                    None => String::from("1999-01-04"),
                },
                Err(_e) => String::from("1999-01-04"), // start of the time series
            };
        let today: String = Zoned::now().date().strftime(DATE_FORMAT).to_string();
        println!("ld: {}, today: {}", last_date, today);
        if last_date >= today {
            return Ok(());
        }

        let mut transaction = pool.begin().await?;

        for other_currency in Currency::iter() {
            if other_currency == BASE_CURRENCY {
                continue;
            }
            let mut currency_from: String = BASE_CURRENCY.to_string();
            let mut currency_to: String = other_currency.to_string();
            let start_date: String = Date::strptime("%Y-%m-%d", &last_date).unwrap().tomorrow().unwrap().strftime("%Y-%m-%d").to_string();
            let url = format!(
                "https://data-api.ecb.europa.eu/service/data/EXR/D.{}.{}.SP00.A?format=csvdata&detail=dataonly&startPeriod={}",
                currency_to,
                currency_from,
                start_date
            );

            let response = reqwest::get(url).await.unwrap();
            let csv_data = response.bytes().await.unwrap();
            let cursor = Cursor::new(csv_data);

            let mut reader = csv::Reader::from_reader(cursor);

            let mut value: f64 = 1.0;
            for record in reader.deserialize() {
                let ecb_record: ECBRecord = record.unwrap();
                let record_date: Date =
                    Date::strptime("%Y-%m-%d", ecb_record.date.as_str()).unwrap();
                let mut dates: Vec<Date> = vec![record_date];
                if record_date.weekday() == Weekday::Friday {
                    dates.push(record_date.tomorrow().unwrap());
                    dates.push(record_date.tomorrow().unwrap().tomorrow().unwrap());
                }
                value = ecb_record.value.unwrap_or(value);

                for date in dates {
                    for _ in 0..2 {
                        // add from->to value, and then add to->from 1/value
                        let date_string: String = date.strftime(DATE_FORMAT).to_string();
                        let id: String =
                            format!("{}_{}_{}", date_string, currency_from, currency_to,);
                        sqlx::query_file!(
                            "src/queries/insertion/insert_into_currency_exchanges.sql",
                            id,
                            date_string,
                            currency_from,
                            currency_to,
                            value,
                        )
                        .execute(&mut *transaction)
                        .await?;

                        let temp: String = currency_from;
                        currency_from = currency_to;
                        currency_to = temp;
                        value = 1.0 / value;
                    }
                }
            }
        }
        transaction.commit().await?;

        Ok(())
    }

    pub(crate) async fn init(
        financial_database_url: &str,
    ) -> Result<FinancialDataBase, sqlx::Error> {
        let pool: SqlitePool = match Sqlite::database_exists(financial_database_url).await? {
            true => {
                println!("Connecting to existing database!");
                SqlitePool::connect(financial_database_url).await?
            }
            false => {
                println!("Creating new database!");
                FinancialDataBase::first_boot(financial_database_url).await?
            }
        };

        FinancialDataBase::init_currency_exchange(&pool).await?;

        Ok(FinancialDataBase { pool })
    }
}

impl Default for FinancialDataBase {
    fn default() -> Self {
        Runtime::new().unwrap().block_on(async {
            Self::init(FINANCIAL_DATABASE_URL)
                .await
                .expect("Pray Tux this never happens.")
        })
    }
}
