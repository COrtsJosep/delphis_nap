use crate::financial::*;
use crate::table_records::*;
use chrono::Local;
use sqlx::sqlite::SqliteConnection;
use sqlx::Connection;
use std::path::Path;
use tokio::fs;

const FINANCIAL_DATABASE_URL: &str = "sqlite://./data/financial_database.sqlite";
const DATE_FORMAT: &str = "%Y-%m-%d";

pub struct FinancialDataBase {
    connection: SqliteConnection,
}

impl FinancialDataBase {
    async fn new() -> Result<FinancialDataBase, sqlx::Error> {
        let financial_database_path_str = FINANCIAL_DATABASE_URL.strip_prefix("sqlite://").unwrap();
        let financial_database_path = Path::new(financial_database_path_str);

        fs::File::create_new(financial_database_path).await.expect("Attempted to create new SQLite database, but there already exists one! This should never happen.");
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

        Ok(FinancialDataBase { connection })
    }

    pub(crate) async fn init() -> Result<FinancialDataBase, sqlx::Error> {
        let financial_database_path_str = FINANCIAL_DATABASE_URL.strip_prefix("sqlite://").unwrap();
        let financial_database_path = Path::new(financial_database_path_str);

        if financial_database_path.exists() {
            //FinancialDataBase::new().await
            println!("Connecting to existing database");
            let connection: SqliteConnection =
                SqliteConnection::connect(FINANCIAL_DATABASE_URL).await?;
            Ok(FinancialDataBase { connection })
        } else {
            println!("Creating new database");
            FinancialDataBase::new().await
        }

        // TODO: currency exchange tables
    }

    pub(crate) async fn insert_account(
        &mut self,
        account: &mut Account,
    ) -> Result<(), sqlx::Error> {
        let query_result = sqlx::query!("select max(account_id) as max_account_id from accounts")
            .fetch_one(&mut self.connection)
            .await;
        let account_id: i64 = match query_result {
            Ok(id) => id.max_account_id.unwrap() + 1i64,
            Err(_e) => 0i64,
        };

        let account_name: String = account.name();
        let account_country: String = account.country();
        let account_currency: String = account.currency().to_string();
        let account_type: String = account.account_type().to_string();
        let account_initial_balance: f64 = account.initial_balance();
        let account_creation_date: String =
            Local::now().date_naive().format(DATE_FORMAT).to_string();
        sqlx::query_file!(
            "src/queries/insertion/insert_into_accounts.sql",
            account_id,
            account_name,
            account_country,
            account_currency,
            account_type,
            account_initial_balance,
            account_creation_date,
        )
        .execute(&mut self.connection)
        .await?;

        Ok(())
    }

    pub(crate) async fn insert_entity(&mut self, entity: &mut Entity) -> Result<(), sqlx::Error> {
        let query_result = sqlx::query!("select max(entity_id) as max_entity_id from entities")
            .fetch_one(&mut self.connection)
            .await;
        let entity_id: i64 = match query_result {
            Ok(id) => id.max_entity_id.unwrap() + 1i64,
            Err(_e) => 0i64,
        };

        let entity_name: String = entity.name();
        let entity_country: String = entity.country();
        let entity_type: String = entity.entity_type().to_string();
        let entity_subtype: String = entity.entity_subtype();
        let entity_creation_date: String =
            Local::now().date_naive().format(DATE_FORMAT).to_string();
        sqlx::query_file!(
            "src/queries/insertion/insert_into_entities.sql",
            entity_id,
            entity_name,
            entity_country,
            entity_type,
            entity_subtype,
            entity_creation_date,
        )
        .execute(&mut self.connection)
        .await?;

        Ok(())
    }

    pub(crate) async fn insert_party(&mut self, party: &mut Party) -> Result<(), sqlx::Error> {
        let query_result = sqlx::query!("select max(party_id) as max_party_id from parties")
            .fetch_one(&mut self.connection)
            .await;
        let party_id: i64 = match query_result {
            Ok(id) => id.max_party_id.unwrap() + 1i64,
            Err(_e) => 0i64,
        };

        let party_creation_date: String = party.creation_date.format(DATE_FORMAT).to_string();
        sqlx::query_file!(
            "src/queries/insertion/insert_into_parties.sql",
            party_id,
            party_creation_date
        )
        .execute(&mut self.connection)
        .await?;

        for transaction in party.iter() {
            self.insert_transaction(&transaction, party_id).await?;
        }

        Ok(())
    }

    // sorry for the long method
    async fn insert_transaction(
        &mut self,
        transaction: &Transaction,
        party_id: i64,
    ) -> Result<(), sqlx::Error> {
        match transaction {
            Transaction::Expense {
                value,
                currency,
                date,
                category,
                subcategory,
                description,
                entity_id,
            } => {
                let query_result =
                    sqlx::query!("select max(expense_id) as max_expense_id from expenses")
                        .fetch_one(&mut self.connection)
                        .await;
                let expense_id: i64 = match query_result {
                    Ok(id) => id.max_expense_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let expense_date: String = date.format(DATE_FORMAT).to_string();
                let expense_currency: String = currency.to_string();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_expenses.sql",
                    expense_id,
                    value,
                    expense_currency,
                    expense_date,
                    category,
                    subcategory,
                    description,
                    entity_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
            Transaction::Income {
                value,
                currency,
                date,
                category,
                subcategory,
                description,
                entity_id,
            } => {
                let query_result =
                    sqlx::query!("select max(income_id) as max_income_id from incomes")
                        .fetch_one(&mut self.connection)
                        .await;
                let income_id: i64 = match query_result {
                    Ok(id) => id.max_income_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let income_date: String = date.format(DATE_FORMAT).to_string();
                let income_currency: String = currency.to_string();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_incomes.sql",
                    income_id,
                    value,
                    income_currency,
                    income_date,
                    category,
                    subcategory,
                    description,
                    entity_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
            Transaction::Credit {
                value,
                currency,
                date,
                account_id,
            } => {
                let query_result = sqlx::query!(
                    "select max(fund_movement_id) as max_fund_movement_id from fund_movements"
                )
                .fetch_one(&mut self.connection)
                .await;
                let fund_movement_id: i64 = match query_result {
                    Ok(id) => id.max_fund_movement_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let fund_movement_date: String = date.format(DATE_FORMAT).to_string();
                let fund_movement_currency: String = currency.to_string();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_fund_movements.sql",
                    fund_movement_id,
                    "Credit",
                    value,
                    fund_movement_currency,
                    fund_movement_date,
                    account_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
            Transaction::Debit {
                value,
                currency,
                date,
                account_id,
            } => {
                let query_result = sqlx::query!(
                    "select max(fund_movement_id) as max_fund_movement_id from fund_movements"
                )
                .fetch_one(&mut self.connection)
                .await;
                let fund_movement_id: i64 = match query_result {
                    Ok(id) => id.max_fund_movement_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let fund_movement_date: String = date.format(DATE_FORMAT).to_string();
                let fund_movement_currency: String = currency.to_string();
                let fund_movement_value: f64 = -1.0 * value;
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_fund_movements.sql",
                    fund_movement_id,
                    "Debit",
                    fund_movement_value,
                    fund_movement_currency,
                    fund_movement_date,
                    account_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
        }
        Ok(())
    }
}
