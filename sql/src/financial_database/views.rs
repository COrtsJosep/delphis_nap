use crate::financial_database::FinancialDataBase;

struct TransactionView {
    transaction_type: String,
    date: String,
    value: f64,
    currency: String,
    name: String,
    category: String,
    subcategory: String,
    party_id: i64,
}

struct FundMovementView {
    fund_movement_type: String,
    value: f64,
    currency: String,
    date: String,
    party_id: i64,
    name: String,
}

impl FinancialDataBase {
    /// Returns a csv in String format with the last n transactions.
    pub(crate) async fn last_transactions(
        &mut self,
        n: i64,
    ) -> Result<Vec<TransactionView>, sqlx::Error> {
        sqlx::query_file_as!(
            TransactionView,
            "src/queries/views/view_last_transactions.sql",
            n
        )
        .fetch_all(&mut self.connection)
        .await
    }

    /// Returns a csv in String format with the last n fund movements.
    pub(crate) async fn last_fund_movements(
        &mut self,
        n: i64,
        account_id: i64,
    ) -> Result<Vec<FundMovementView>, sqlx::Error> {
        if account_id >= 0 {
            sqlx::query_file_as!(
                FundMovementView,
                "src/queries/views/view_last_fund_movements_filtered.sql",
                account_id,
                n
            )
            .fetch_all(&mut self.connection)
            .await
        } else {
            sqlx::query_file_as!(
                FundMovementView,
                "src/queries/views/view_last_fund_movements_unfiltered.sql",
                n
            )
            .fetch_all(&mut self.connection)
            .await
        }
    }
}
