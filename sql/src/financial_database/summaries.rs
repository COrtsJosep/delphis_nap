use crate::financial::Currency;
use crate::financial_database::{FinancialDataBase, DATE_FORMAT};
use chrono::NaiveDate;
use sqlx::Connection;
use std::fmt::Display;
use strum_macros::{EnumIter, EnumString};

struct CurrentFundStandSummary {
    name: String,
    country: String,
    currency: String,
    account_type: String,
    current_value: f64,
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

    pub(crate) async fn current_fund_stand(
        &mut self,
        currency_to: Option<&Currency>,
    ) -> Result<Vec<CurrentFundStandSummary>, sqlx::Error> {
        if let Some(currency_to) = currency_to {
            match currency_to {
                Currency::EUR => {
                    sqlx::query_file_as!(
                        CurrentFundStandSummary,
                        "src/queries/summaries/summary_current_fund_stand_eur.sql"
                    )
                    .fetch_all(&mut self.connection)
                    .await
                }
                Currency::CHF => {
                    sqlx::query_file_as!(
                        CurrentFundStandSummary,
                        "src/queries/summaries/summary_current_fund_stand_chf.sql"
                    )
                    .fetch_all(&mut self.connection)
                    .await
                }
                Currency::SEK => {
                    sqlx::query_file_as!(
                        CurrentFundStandSummary,
                        "src/queries/summaries/summary_current_fund_stand_sek.sql"
                    )
                    .fetch_all(&mut self.connection)
                    .await
                }
            }
        } else {
            sqlx::query_file_as!(
                CurrentFundStandSummary,
                "src/queries/summaries/summary_current_fund_stand_nocurrency.sql"
            )
            .fetch_all(&mut self.connection)
            .await
        }
    }
}
