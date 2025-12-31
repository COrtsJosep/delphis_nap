use crate::modules::financial::Currency;
use chrono::{Local, NaiveDate};
use polars::prelude::*;
use reqwest;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::iter::zip;
use std::str::FromStr;
use std::string::String;
use strum::IntoEnumIterator;

const BASE_CURRENCY: Currency = Currency::EUR;

pub enum Extremum {
    MIN,
    MAX,
}

pub struct CurrencyExchange {
    hash_map_raw: HashMap<String, DataFrame>,
    hash_map: HashMap<String, DataFrame>,
}

impl CurrencyExchange {
    /// Downloads the exchange rate table from the ECB, optionally from a starting date.
    pub(crate) fn download(
        currency: &Currency,
        from_date: Option<NaiveDate>,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let start_period_string: String = match from_date {
            Some(date) => format!("&startPeriod={}", date.to_string()),
            None => String::default(),
        };

        let url: String = format!("https://data-api.ecb.europa.eu/service/data/EXR/D.{}.{}.SP00.A?format=csvdata&detail=dataonly{}",
            currency, BASE_CURRENCY, start_period_string
        );

        let response = reqwest::blocking::get(url)?;
        let csv_data = response.bytes()?;

        let cursor = Cursor::new(csv_data);
        let data_frame: DataFrame = CsvReadOptions::default()
            .with_infer_schema_length(None)
            .with_has_header(true)
            .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
            .into_reader_with_file_handle(cursor)
            .finish()?
            .lazy()
            .select([
                col("TIME_PERIOD").alias("date"),
                (lit(1.0) / col("OBS_VALUE")).alias("value"), // needed because ECB returns foreign in terms of EUR
            ])
            .collect()?;

        Ok(data_frame.to_owned())
    }

    /// Tries to read the exchange rate table from the expected path. If it's there, it is loaded,
    /// and if not up-to-date, it is enhanced with fresh data from the ECB.
    fn load(currency: &Currency) -> Result<DataFrame, Box<dyn std::error::Error>> {
        // could be refactored
        let key: String = CurrencyExchange::key(currency, &BASE_CURRENCY);

        let mut data_frame = CsvReadOptions::default()
            .with_infer_schema_length(None)
            .with_has_header(true)
            .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
            .try_into_reader_with_file_path(Some(format!("data/exchange_rate_{}.csv", key).into()))?
            .finish()?;

        let extremum: Extremum = Extremum::MAX;
        let max_date: NaiveDate = Self::extreme_date(&data_frame, &extremum)?;
        if max_date < Local::now().date_naive() {
            let new_records: DataFrame = Self::download(currency, Option::from(max_date))?;
            data_frame = data_frame.vstack(&new_records)?
        }

        Ok(data_frame)
    }

    /// Adds any missing days and fills them with a forward rolling strategy.
    fn expand(original_data_frame: &DataFrame, add_today: bool) -> Result<DataFrame, PolarsError> {
        let mut data_frame: DataFrame = original_data_frame.clone();

        if add_today {
            let extremum: Extremum = Extremum::MAX;
            if Self::extreme_date(&data_frame, &extremum)? < Local::now().date_naive() {
                let extra_row: DataFrame = df!(
                    "date" => [Local::now().date_naive()],
                    "value" => [None::<f64>]
                )?;

                data_frame = data_frame.vstack(&extra_row)?;
            }
        }

        Ok(data_frame
            .upsample::<[String; 0]>([], "date", Duration::parse("1d"))?
            .fill_null(FillNullStrategy::Forward(None))?)
    }

    #[cfg(test)]
    pub(crate) fn test_expand(
        data_frame: &DataFrame,
        add_today: bool,
    ) -> Result<DataFrame, PolarsError> {
        Self::expand(&data_frame, add_today)
    }

    /// Returns the most recent date of the dataframe's column "date".
    /// Unsure if propperly written.
    fn extreme_date(data_frame: &DataFrame, extrema: &Extremum) -> Result<NaiveDate, PolarsError> {
        let iterator = data_frame.column("date")?.date()?.as_date_iter();

        let result: Option<Option<NaiveDate>> = match extrema {
            Extremum::MIN => iterator.min(),
            Extremum::MAX => iterator.max(),
        };

        // Flatten the Option<Option<NaiveDate>> and convert None to Err
        result.flatten().ok_or_else(|| {
            PolarsError::ComputeError("Column is empty or contains no valid dates!".into())
        })
    }

    #[cfg(test)] // public only for testing
    pub(crate) fn test_extreme_date(
        data_frame: &DataFrame,
        extremum: &Extremum,
    ) -> Result<NaiveDate, PolarsError> {
        Self::extreme_date(data_frame, extremum)
    }

    #[cfg(test)]
    pub(crate) fn new(
        hash_map_raw: HashMap<String, DataFrame>,
    ) -> Result<CurrencyExchange, PolarsError> {
        let mut hash_map: HashMap<String, DataFrame> = HashMap::new();
        for (key, data_frame) in hash_map_raw.iter() {
            hash_map.insert(key.to_owned(), Self::expand(data_frame, true)?);
        }
        Ok(CurrencyExchange {
            hash_map_raw,
            hash_map,
        })
    }

    /// Initializes the currency exchange module
    pub(crate) fn init() -> Result<CurrencyExchange, Box<dyn std::error::Error>> {
        let mut hash_map_raw: HashMap<String, DataFrame> = HashMap::new();
        let mut hash_map: HashMap<String, DataFrame> = HashMap::new();

        for currency in Currency::iter() {
            if currency == BASE_CURRENCY {
                continue;
            }

            let key: String = CurrencyExchange::key(&currency, &BASE_CURRENCY);
            let data_frame: DataFrame = match Self::load(&currency) {
                Ok(data_frame) => data_frame,
                Err(_err) => Self::download(&currency, None)?,
            };

            let mut expanded_data_frame: DataFrame = data_frame.clone();
            expanded_data_frame = Self::expand(&mut expanded_data_frame, true)?;

            let key_raw = key.clone();
            hash_map_raw.insert(key_raw, data_frame);
            hash_map.insert(key, expanded_data_frame);
        }

        let mut currency_exchange = CurrencyExchange {
            hash_map_raw,
            hash_map,
        };
        currency_exchange.save();

        Ok(currency_exchange)
    }

    /// Saves the currency exchange tables.
    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (key, data_frame) in self.hash_map_raw.iter_mut() {
            if data_frame.is_empty() {
                return Ok(());
            }

            let mut file = File::create(format!("data/exchange_rate_{}.csv", key))?;

            CsvWriter::new(&mut file)
                .include_header(true)
                .with_separator(b',')
                .finish(data_frame)?;
        }

        Ok(())
    }

    /// Creates a key for the currency exchange HashMap (just the concatenation of the currency
    /// names)
    fn key(currency_from: &Currency, currency_to: &Currency) -> String {
        format!("{}{}", currency_from, currency_to)
    }

    /// Returns the historic exchange rate between two currencies at a given date
    pub(crate) fn exchange_currency(
        &self,
        currency_from: &Currency,
        currency_to: &Currency,
        date: NaiveDate,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        if currency_to == currency_from {
            return Ok(1.0);
        }

        let key: String = CurrencyExchange::key(currency_from, currency_to);
        let inverse_key: String = CurrencyExchange::key(currency_to, currency_from);
        if self.hash_map.contains_key(&key) {
            let data_frame = self.hash_map.get(&key).ok_or("No key!")?;
            assert!(
                Self::extreme_date(data_frame, &Extremum::MIN)? <= date,
                "Tried to get an exchange rate too far away in the past"
            );
            assert!(
                date <= Self::extreme_date(data_frame, &Extremum::MAX)?,
                "Tried to get an exchange rate too near the present"
            );

            Ok(data_frame
                .clone()
                .lazy()
                .filter(col("date").eq(lit(date)))
                .collect()?
                .column("value")?
                .f64()?
                .get(0)
                .ok_or_else(|| PolarsError::NoData("Column is empty!".into()))?)
        } else if self.hash_map.contains_key(&inverse_key) {
            Ok(1.0 / self.exchange_currency(currency_to, currency_from, date)?)
        } else {
            Ok(self.exchange_currency(currency_from, &BASE_CURRENCY, date)?
                * self.exchange_currency(&BASE_CURRENCY, currency_to, date)?)
        }
    }

    #[cfg(test)]
    pub(crate) fn test_exchange_currency(
        &self,
        currency_from: &Currency,
        currency_to: &Currency,
        date: NaiveDate,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        self.exchange_currency(currency_from, currency_to, date)
    }

    /// This function takes a dataframe with columns date, currency, and value, and returns a
    /// dataframe with the same columns, but the value has been converted to the currency_to
    pub(crate) fn exchange_currencies(
        &self,
        currency_to: &Currency,
        data_frame: DataFrame,
    ) -> Result<DataFrame, Box<dyn std::error::Error>> {
        let mut exchange_rates: Vec<f64> = vec![];
        let date_iter = data_frame.column("date")?.date()?.as_date_iter();
        let currency_iter = data_frame.column("currency")?.str()?.into_iter();

        for (date, currency) in zip(date_iter, currency_iter) {
            let date = date.ok_or("No date!")?;
            let currency_from = Currency::from_str(currency.ok_or("No currency!")?)?;
            let exchange_rate = self.exchange_currency(&currency_from, &currency_to, date)?;

            exchange_rates.push(exchange_rate);
        }

        Ok(data_frame
            .lazy()
            .with_column(lit(Series::new("exchange_rate".into(), exchange_rates)))
            .with_column((col("exchange_rate") * col("value")).alias("value"))
            .drop(["exchange_rate", "currency"])
            .collect()?)
    }
}
