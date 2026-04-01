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

        sqlx::query_file!("src/queries/table_creation/create_party_table.sql")
            .execute(&mut connection)
            .await?;

        Ok(FinancialDataBase { connection })
    }

    pub(crate) async fn init() -> Result<FinancialDataBase, sqlx::Error> {
        let financial_database_path_str = FINANCIAL_DATABASE_URL.strip_prefix("sqlite://").unwrap();
        let financial_database_path = Path::new(financial_database_path_str);

        if financial_database_path.exists() {
            println!("here!");
            let connection: SqliteConnection =
                SqliteConnection::connect(FINANCIAL_DATABASE_URL).await?;
            Ok(FinancialDataBase { connection })
        } else {
            FinancialDataBase::new().await
        }
    }
}
