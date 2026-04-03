pub mod ser_deser;

use crate::table_records::*;
use sqlx::sqlite::SqliteConnection;
use sqlx::Connection;
use std::path::Path;
use tokio::fs;

const FINANCIAL_DATABASE_URL: &str = "sqlite://./data/financial_database.sqlite";

pub struct FinancialDataBase {
    connection: SqliteConnection,
}

impl FinancialDataBase {
    async fn first_boot() -> Result<SqliteConnection, sqlx::Error> {
        let financial_database_path_str = FINANCIAL_DATABASE_URL.strip_prefix("sqlite://").unwrap();
        let financial_database_path = Path::new(financial_database_path_str);

        //fs::File::create_new(financial_database_path).await.expect("Attempted to create new SQLite database, but there already exists one! This should never happen.");
        println!("New database created!");
        let mut connection = SqliteConnection::connect(FINANCIAL_DATABASE_URL).await?;

        // initalization of all six tables
        sqlx::query_file!("src/queries/table_creation/create_account_table.sql")
            .execute(&mut connection)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_entity_table.sql")
            .execute(&mut connection)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_expense_table.sql")
            .execute(&mut connection)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_fund_movement_table.sql")
            .execute(&mut connection)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_income_table.sql")
            .execute(&mut connection)
            .await?;
        sqlx::query_file!("src/queries/table_creation/create_party_table.sql")
            .execute(&mut connection)
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
                .execute(&mut connection)
                .await?;
            }
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
                .execute(&mut connection)
                .await?;
            }
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
                .execute(&mut connection)
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
                .execute(&mut connection)
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
                .execute(&mut connection)
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
                .execute(&mut connection)
                .await?;
            }
        }

        Ok(connection)
    }

    //async fn init_currency_exchange(connection: &mut SqliteConnection) -> Result<(), sqlx::Error> {}

    pub(crate) async fn init() -> Result<FinancialDataBase, sqlx::Error> {
        let financial_database_path_str = FINANCIAL_DATABASE_URL.strip_prefix("sqlite://").unwrap();
        let financial_database_path = Path::new(financial_database_path_str);

        let mut connection: SqliteConnection = match financial_database_path.exists() {
            true => {
                println!("Connecting to existing database");
                SqliteConnection::connect(FINANCIAL_DATABASE_URL).await?
            }
            false => {
                println!("Creating new database");
                FinancialDataBase::first_boot().await?
            }
        };

        //FinancialDataBase::init_currency_exchange(&mut connection).await?;

        Ok(FinancialDataBase { connection })
    }
}
