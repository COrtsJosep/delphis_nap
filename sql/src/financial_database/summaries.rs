use crate::financial::Currency;
use crate::financial_database::{FinancialDataBase, DATE_FORMAT};
use chrono::NaiveDate;
use sqlx::{sqlite::SqliteRow, Connection};
use std::fmt::Display;
use strum_macros::{EnumIter, EnumString};

struct CurrentFundStandSummary {
    name: String,
    country: String,
    currency: String,
    account_type: String,
    current_value: f64,
}

struct ExpenseSummaryRow {
    category: String,
    subcategory: String,
    value: f64,
    value_day: f64,
    value_total_expenses: f64,
    value_total_incomes: f64,
}

#[derive(Debug, Hash, PartialEq, Eq, EnumIter, Clone, EnumString)]
pub(crate) enum TimeUnit {
    Day,
    Week,
    Month,
    Year,
}

impl TimeUnit {
    pub(crate) fn duration(&self) -> &str {
        match self {
            TimeUnit::Day => "1d",
            TimeUnit::Week => "1w",
            TimeUnit::Month => "1mo",
            TimeUnit::Year => "1y",
        }
    }
    fn date_format(&self) -> &str {
        match self {
            TimeUnit::Day => "%Y-%m-%d",
            TimeUnit::Week => "%Y-%W",
            TimeUnit::Month => "%Y-%m",
            TimeUnit::Year => "%Y",
        }
    }
}

// Conversion to string
impl Display for TimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TimeUnit::Day => "Day".to_string(),
            TimeUnit::Week => "Week".to_string(),
            TimeUnit::Month => "Month".to_string(),
            TimeUnit::Year => "Year".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Default for TimeUnit {
    fn default() -> Self {
        TimeUnit::Month
    }
}

impl FinancialDataBase {
    /// Calculates the sum of all the incomes earned between date_from to date_to, both included,
    /// in the currency currency_to.
    async fn total_income(
        &mut self,
        date_from: NaiveDate,
        date_to: NaiveDate,
        currency_to: &Currency,
    ) -> Result<f64, sqlx::Error> {
        let date_from_string: String = date_from.format(DATE_FORMAT).to_string();
        let date_to_string: String = date_to.format(DATE_FORMAT).to_string();
        let currency_to_string: String = currency_to.to_string();

        let record = sqlx::query_file!(
            "src/queries/summaries/calculate_total_income.sql",
            date_from_string,
            date_to_string,
            currency_to_string,
        )
        .fetch_one(&mut self.connection)
        .await?;

        Ok(record.total_income.unwrap_or(0.0f64))
    }

    /// check: might the fund_changes.value column be null?
    pub(crate) async fn current_fund_stand(
        &mut self,
        currency_to: Option<&Currency>,
    ) -> Result<Vec<CurrentFundStandSummary>, sqlx::Error> {
        if let Some(currency_to) = currency_to {
            let currency_to_string: String = currency_to.to_string();
            sqlx::query_file_as!(
                CurrentFundStandSummary,
                "src/queries/summaries/summary_current_fund_stand_currency.sql",
                currency_to_string,
                currency_to_string
            )
            .fetch_all(&mut self.connection)
            .await
        } else {
            sqlx::query_file_as!(
                CurrentFundStandSummary,
                "src/queries/summaries/summary_current_fund_stand_nocurrency.sql"
            )
            .fetch_all(&mut self.connection)
            .await
        }
    }

    /// Generates a summary table of all expenses between date_from to date_to, expressed in the currency_to
    pub(crate) async fn expenses_summary(
        &mut self,
        date_from: NaiveDate,
        date_to: NaiveDate,
        currency_to: &Currency,
    ) -> Result<Vec<ExpenseSummaryRow>, sqlx::Error> {
        let total_income: f64 = self.total_income(date_from, date_to, currency_to).await?;
        let num_days: i64 = date_to.signed_duration_since(date_from).num_days();
        let date_from_string: String = date_from.to_string();
        let date_to_string: String = date_to.to_string();
        let currency_to_string: String = currency_to.to_string();

        let mut transaction = self.connection.begin().await?;

        sqlx::query_file!(
            "src/queries/summaries/temporary_expenses_grouped.sql",
            currency_to_string,
            date_from_string,
            date_to_string
        )
        .execute(&mut *transaction)
        .await?;

        let mut expense_summary_rows: Vec<ExpenseSummaryRow> = sqlx::query_file_as!(
            ExpenseSummaryRow,
            "src/queries/summaries/summary_expenses.sql",
            num_days,
            total_income
        )
        .fetch_all(&mut *transaction)
        .await?;

        let expense_summary_last_row: ExpenseSummaryRow = sqlx::query_file_as!(
            ExpenseSummaryRow,
            "src/queries/summaries/summary_expenses_total.sql",
            num_days,
            total_income
        )
        .fetch_one(&mut *transaction)
        .await?;

        expense_summary_rows.push(expense_summary_last_row);

        sqlx::query!("drop table expenses_temporary")
            .execute(&mut *transaction)
            .await?;

        transaction.commit().await?;

        Ok(expense_summary_rows)
    }

    /// sadly, in this case the query needs to be constructed dinamically
    /// because we do not know what columns will be retrieved (since we do
    /// not know what unique categories are stored in the database).
    pub(crate) async fn evolution_table(
        &mut self,
        currency_to: &Currency,
        time_unit: &TimeUnit,
    ) -> Result<(Vec<String>, Vec<SqliteRow>), sqlx::Error> {
        let currency_to_string: String = currency_to.to_string();
        let time_unit_format: String = time_unit.date_format().to_string();
        let unique_categories: Vec<String> = sqlx::query!("select distinct category from expenses")
            .fetch_all(&mut self.connection)
            .await?
            .into_iter()
            .map(|record| record.category)
            .collect::<Vec<String>>();

        sqlx::query_file!(
            "src/queries/summaries/temporary_expense_evolution_grouped.sql",
            time_unit_format,
            currency_to_string,
            time_unit_format,
            time_unit_format
        )
        .execute(&mut self.connection)
        .await?;

        let mut query_string: String = String::from("select date, ");
        for unique_category in &unique_categories {
            let new_line: String = format!(
                "sum(case when category = '{}' then value else 0.0 end) as {},",
                unique_category,
                unique_category.replace(" ", "_")
            );
            query_string = query_string + &new_line;
        }

        query_string = query_string[..(query_string.len() - 1)].to_string() // remove last comma
            + " from expense_evolution_temporary";
        let query_str: &str = &query_string;

        let rows: Vec<SqliteRow> = sqlx::query(query_str)
            .fetch_all(&mut self.connection)
            .await?;

        Ok((unique_categories, rows))
    }
}
