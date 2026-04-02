use crate::table_records::*;
use crate::financial::*;
use sqlx::sqlite::SqliteConnection;
use sqlx::Connection;
use std::path::Path;
use tokio::fs;

const FINANCIAL_DATABASE_URL: &str = "sqlite://./data/financial_database.sqlite";

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
                sqlx::query!(
                    r#"
                    insert into accounts 
                    (account_id, name, country, currency, account_type, initial_balance, creation_date) 
                    values (?, ?, ?, ?, ?, ?, ?)"#,
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
                sqlx::query!(
                    r#"
                    insert into entities
                    (entity_id, name, country, entity_type, entity_subtype, creation_date)
                    values (?, ?, ?, ?, ?, ?)
                    "#,
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
			    sqlx::query!(
			        r#"
			        insert into parties
			        (party_id, creation_date)
			        values (?, ?)
			        "#,
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
			    let expense_record: ExpenseRecord = result.expect("Crashed here");
			    sqlx::query!(
			        r#"
			        insert into expenses
			        (expense_id, value, currency, date, category, subcategory, description, entity_id, party_id)
			        values (?, ?, ?, ?, ?, ?, ?, ?, ?)
			        "#,
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
		    	sqlx::query!(
			        r#"
			        insert into fund_movements
			        (fund_movement_id, fund_movement_type, value, currency, date, account_id, party_id)
			        values (?, ?, ?, ?, ?, ?, ?)
			        "#,
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
			    sqlx::query!(
			        r#"
			        insert into incomes
			        (income_id, value, currency, date, category, subcategory, description, entity_id, party_id)
			        values (?, ?, ?, ?, ?, ?, ?, ?, ?)
			        "#,
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

    pub(crate) async fn insert_party(&mut self, party: &mut Party) -> Result<(), sqlx::Error> {
        let query_result = sqlx::query!("select max(party_id) as max_party_id from parties")
            .fetch_one(&mut self.connection)
            .await;
        let party_id: i64 = match query_result {
            Ok(id) => id.max_party_id.unwrap(),
            Err(_e) => 0i64,
        };

        let party_creation_date: String = party.creation_date.format("%Y-%m-%d").to_string();
        sqlx::query!(
                "insert into parties (party_id, creation_date) values (?, ?)", 
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

    async fn insert_transaction(
        &mut self,
        transaction: &Transaction,
        party_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("insert into parties (party_id, creation_date) values (2, '2020-02-02')")
            .execute(&mut self.connection)
            .await?;

        Ok(())
    }
}
