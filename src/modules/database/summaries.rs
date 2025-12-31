use crate::modules::currency_exchange::CurrencyExchange;
use crate::modules::database::{capitalize_every_word, data_frame_to_csv_string, DataBase};
use crate::modules::financial::Currency;
use chrono::{Local, NaiveDate};
use polars::prelude::pivot::pivot_stable;
use polars::prelude::*;
use std::fmt::Display;
use std::str::FromStr;
use strum_macros::{EnumIter, EnumString};

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

impl DataBase {
    /// Calculates the sum of all the incomes earned between date_from to date_to, both included,
    /// in the currency currency_to.
    fn total_income(
        &self,
        date_from: NaiveDate,
        date_to: NaiveDate,
        currency_to: &Currency,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let currency_exchange: CurrencyExchange = CurrencyExchange::init()?;

        let income_table: DataFrame = self
            .incomes_table
            .data_frame
            .clone()
            .lazy()
            .filter(col("date").is_between(lit(date_from), lit(date_to), ClosedInterval::Both))
            .collect()?;

        let mut exchange_rates = Vec::new();
        let currency_iterator = income_table.column("currency")?.str()?.into_iter();
        for currency in currency_iterator {
            let currency_from = Currency::from_str(currency.ok_or("Null in currency column!")?)?;
            let exchange_rate: f64 =
                currency_exchange.exchange_currency(&currency_from, currency_to, date_to)?;
            exchange_rates.push(exchange_rate);
        }

        let exchange_rates: Series = Series::new("exchange_rate".into(), exchange_rates);

        Ok(income_table
            .lazy()
            .with_column(exchange_rates.lit())
            .with_column((col("exchange_rate") * col("value")).alias(currency_to.to_string()))
            .collect()?
            .column(currency_to.to_string().as_str())?
            .f64()?
            .sum()
            .ok_or("Sum of empty values?")?)
    }

    pub(crate) fn current_fund_stand(
        &self,
        currency_to: Option<&Currency>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let currency_exchange: CurrencyExchange = CurrencyExchange::init()?;

        let initial_balances: DataFrame = self.account_table.data_frame.clone();

        let funds_table: DataFrame = self
            .funds_table
            .data_frame
            .clone()
            .lazy()
            .group_by(["account_id", "currency"])
            .agg([col("value").sum()])
            .collect()?;

        let mut summary = initial_balances
            .lazy()
            .join(
                funds_table.clone().lazy(),
                [col("account_id"), col("currency")],
                [col("account_id"), col("currency")],
                JoinArgs::new(JoinType::Left),
            )
            .with_column(col("value").fill_null(0.0))
            .with_column((col("initial_balance") + col("value")).alias("total_value"))
            .collect()?;

        if let Some(currency_to) = currency_to {
            let mut exchange_rates = Vec::new();
            let currency_iterator = summary.column("currency")?.str()?.into_iter();
            for currency in currency_iterator {
                let currency_from =
                    Currency::from_str(currency.ok_or("Null value in currency column!")?)?;
                let exchange_rate: f64 = currency_exchange.exchange_currency(
                    &currency_from,
                    currency_to,
                    Local::now().date_naive(),
                )?;
                exchange_rates.push(exchange_rate);
            }

            let exchange_rates: Series = Series::new("exchange_rate".into(), exchange_rates);

            summary = summary
                .lazy()
                .with_column(exchange_rates.lit())
                .with_column(
                    (col("exchange_rate") * col("total_value")).alias(currency_to.to_string()),
                )
                .group_by(["name", "country", "account_type"])
                .agg([col(currency_to.to_string().as_str()).sum()])
                .sort(
                    [currency_to.to_string()],
                    SortMultipleOptions::default().with_order_descending(true),
                )
                .select([
                    col("name"),
                    col("country"),
                    col("account_type"),
                    col(currency_to.to_string()).round(2),
                ])
                .filter(col(currency_to.to_string()).gt_eq(lit(0.01)))
                .select([all().name().map(|name| {
                    Ok(PlSmallStr::from_string(capitalize_every_word(
                        name.replace("_", " "),
                    )))
                })])
                .collect()?
        } else {
            summary = summary
                .clone()
                .lazy()
                .sort(
                    ["currency", "total_value"],
                    SortMultipleOptions::default().with_order_descending_multi([false, true]),
                )
                .select([
                    col("name"),
                    col("country"),
                    col("currency"),
                    col("account_type"),
                    col("total_value").round(2),
                ])
                .filter(col("total_value").abs().gt_eq(lit(0.01)))
                .select([all().name().map(|name| {
                    Ok(PlSmallStr::from_string(capitalize_every_word(
                        name.replace("_", " "),
                    )))
                })])
                .collect()?
        }

        data_frame_to_csv_string(&mut summary)
    }

    /// Generates a summary table of all expenses between date_from to date_to, expressed in the currency_to
    pub(crate) fn expenses_summary(
        &self,
        date_from: NaiveDate,
        date_to: NaiveDate,
        currency_to: &Currency,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let currency_exchange: CurrencyExchange = CurrencyExchange::init()?;
        let total_income: f64 = self.total_income(date_from, date_to, currency_to)?;
        let num_days: i64 = date_to.signed_duration_since(date_from).num_days();

        let expenses_table: DataFrame = self
            .expenses_table
            .data_frame
            .clone()
            .lazy()
            .filter(col("date").is_between(lit(date_from), lit(date_to), ClosedInterval::Both))
            .collect()?;

        let mut exchange_rates = Vec::new();
        let currency_iterator = expenses_table.column("currency")?.str()?.into_iter();
        for currency in currency_iterator {
            let currency_from =
                Currency::from_str(currency.ok_or("Null value in currency column!")?)?;
            let exchange_rate: f64 =
                currency_exchange.exchange_currency(&currency_from, currency_to, date_to)?;
            exchange_rates.push(exchange_rate);
        }

        let exchange_rates: Series = Series::new("exchange_rate".into(), exchange_rates);

        let mut summary: DataFrame = expenses_table
            .lazy()
            .with_column(exchange_rates.lit())
            .with_column((col("exchange_rate") * col("value")).alias(currency_to.to_string()))
            .group_by([col("category"), col("subcategory")])
            .agg([col(currency_to.to_string()).sum()])
            .with_columns([
                col(currency_to.to_string()).round(2),
                (col(currency_to.to_string()) / lit(num_days))
                    .round(2)
                    .alias(format!("{}_/_day", currency_to.to_string()).as_str()),
                (col(currency_to.to_string()) * lit(100) / col(currency_to.to_string()).sum())
                    .round(2)
                    .alias("%_total_expenses"),
                (col(currency_to.to_string()) * lit(100) / lit(total_income))
                    .round(2)
                    .alias("%_total_income"),
            ])
            .sort(
                ["category", "subcategory"],
                SortMultipleOptions::default().with_order_descending_multi([false, false]),
            )
            .select([all().name().map(|name| {
                Ok(PlSmallStr::from_string(capitalize_every_word(
                    name.replace("_", " "),
                )))
            })])
            .collect()?;

        let total_expenses: f64 = summary
            .column(currency_to.to_string().as_str())?
            .f64()?
            .sum()
            .ok_or("Sum of null values?")?;

        let last_row: DataFrame = df!(
        "Category" => ["Total"],
        "Subcategory" => ["Total"],
        currency_to.to_string().as_str() => [(100.0 * total_expenses).round() / 100.0],
        format!("{} / Day", currency_to.to_string()).as_str() => [(100.0 * total_expenses / num_days as f64).round() / 100.0],
        "% Total Expenses" => [100.0],
        "% Total Income" => [(100.0 * 100.0 * total_expenses / total_income).round() / 100.0]
        )?;

        summary = summary.vstack(&last_row)?;

        data_frame_to_csv_string(&mut summary)
    }

    pub(crate) fn evolution_table(
        &self,
        currency_to: &Currency,
        time_unit: &TimeUnit,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let currency_exchange: CurrencyExchange = CurrencyExchange::init()?;
        let duration: &str = time_unit.duration();

        let expenses_table: DataFrame = self.expenses_table.data_frame.clone();

        let mut exchange_rates = Vec::new();
        let currency_iterator = expenses_table.column("currency")?.str()?.into_iter();
        for currency in currency_iterator {
            let currency_from =
                Currency::from_str(currency.ok_or("Null value in the currency column!")?)?;
            let exchange_rate: f64 = currency_exchange.exchange_currency(
                &currency_from,
                currency_to,
                Local::now().date_naive(),
            )?;
            exchange_rates.push(exchange_rate);
        }

        let exchange_rates: Series = Series::new("exchange_rate".into(), exchange_rates);

        let summary: DataFrame = expenses_table
            .lazy()
            .with_column(exchange_rates.lit())
            .with_column((col("exchange_rate") * col("value")).alias(currency_to.to_string()))
            .sort(["date"], Default::default())
            .group_by_dynamic(
                col("date"),
                [col("category")],
                DynamicGroupOptions {
                    every: Duration::parse(duration),
                    period: Duration::parse(duration),
                    offset: Duration::parse("0"),
                    ..Default::default()
                },
            )
            .agg([col(currency_to.to_string()).sum().round(2)])
            .collect()?;

        let mut pivoted_summary: DataFrame = pivot_stable(
            &summary,
            ["category"],
            Some(["date"]),
            Some([currency_to.to_string()]),
            true,
            None,
            None,
        )?
        .lazy()
        .sort(["date"], Default::default())
        .collect()?
        .upsample::<[String; 0]>([], "date", Duration::parse(duration))?
        .fill_null(FillNullStrategy::Zero)?;

        pivoted_summary.rename("date", PlSmallStr::from_string(time_unit.to_string()))?;

        data_frame_to_csv_string(&mut pivoted_summary)
    }
}
