use sqlx::mysql::MySqlConnection;
use sqlx::Connection;

pub struct FinancialDataBase {
    connection: MySqlConnection,
}

impl FinancialDataBase {
    #[tokio::main]
    pub(crate) async fn new() -> Result<FinancialDataBase, sqlx::Error> {
        let connection = MySqlConnection::connect("sqlite::memory:").await?;
        Ok(FinancialDataBase { connection })
    }
}
