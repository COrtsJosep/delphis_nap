#[cfg(test)]
#[path = "tests.rs"]
mod tests {
    use crate::modules::currency_exchange::{CurrencyExchange, Extremum};
    use crate::modules::financial::Currency;
    use chrono::NaiveDate;
    use polars::prelude::*;
    use std::collections::HashMap;

    fn init_testing_currency_exchange() -> CurrencyExchange {
        let data_frame_chfeur: DataFrame = df!(
            "date" => [
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 3).unwrap(),
            ],
            "value" => [0.5, 1.0, 1.5]
        )
        .unwrap();

        let data_frame_sekeur: DataFrame = df!(
            "date" => [
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 3).unwrap(),
            ],
            "value" => [1.5, 1.0, 0.5]
        )
        .unwrap();

        let mut hash_map = HashMap::new();
        hash_map.insert(String::from("CHFEUR"), data_frame_chfeur);
        hash_map.insert(String::from("SEKEUR"), data_frame_sekeur);

        CurrencyExchange::new(hash_map).unwrap()
    }
    #[test]
    fn correct_max_date() {
        let extremum: Extremum = Extremum::MAX;
        let max_date: NaiveDate = NaiveDate::from_ymd_opt(1997, 1, 10).unwrap();
        let data_frame: DataFrame = df!(
            "date" => [
                NaiveDate::from_ymd_opt(1985, 2, 15).unwrap(),
                max_date,
                NaiveDate::from_ymd_opt(1983, 3, 22).unwrap(),
                NaiveDate::from_ymd_opt(1981, 4, 30).unwrap(),
            ],
            "value" => [57.9, 72.5, 53.6, 83.1]
        )
        .unwrap();

        assert_eq!(
            CurrencyExchange::test_extreme_date(&data_frame, &extremum).unwrap(),
            max_date
        )
    }

    #[test]
    fn correct_expand() {
        let mut data_frame: DataFrame = df!(
            "date" => [
                NaiveDate::from_ymd_opt(1985, 2, 15).unwrap(),
                NaiveDate::from_ymd_opt(1985, 2, 17).unwrap(),
                NaiveDate::from_ymd_opt(1985, 2, 18).unwrap(),
            ],
            "value" => [57.9, 72.5, 83.1]
        )
        .unwrap();
        data_frame = CurrencyExchange::test_expand(&data_frame, false).unwrap();

        let expanded_data_frame = df!(
            "date" => [
                NaiveDate::from_ymd_opt(1985, 2, 15).unwrap(),
                NaiveDate::from_ymd_opt(1985, 2, 16).unwrap(),
                NaiveDate::from_ymd_opt(1985, 2, 17).unwrap(),
                NaiveDate::from_ymd_opt(1985, 2, 18).unwrap(),
            ],
            "value" => [57.9, 57.9, 72.5, 83.1]
        )
        .unwrap();

        assert!(data_frame.equals(&expanded_data_frame))
    }

    #[test]
    fn correct_direct_exchange() {
        let currency_exchange: CurrencyExchange = init_testing_currency_exchange();

        let currency_from: Currency = Currency::CHF;
        let currency_to: Currency = Currency::EUR;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        assert_eq!(
            currency_exchange
                .test_exchange_currency(&currency_from, &currency_to, date)
                .unwrap(),
            0.5
        );
    }

    #[test]
    fn correct_inverse_exchange() {
        let currency_exchange: CurrencyExchange = init_testing_currency_exchange();

        let currency_from: Currency = Currency::EUR;
        let currency_to: Currency = Currency::CHF;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        assert_eq!(
            currency_exchange
                .test_exchange_currency(&currency_from, &currency_to, date)
                .unwrap(),
            1.0 / 0.5
        );
    }

    #[test]
    fn correct_bridged_exchange() {
        let currency_exchange: CurrencyExchange = init_testing_currency_exchange();

        let currency_from: Currency = Currency::CHF;
        let currency_to: Currency = Currency::SEK;
        let date: NaiveDate = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        assert_eq!(
            currency_exchange
                .test_exchange_currency(&currency_from, &currency_to, date)
                .unwrap(),
            0.5 / 1.5
        );
    }

    #[test]
    fn correct_dataframe_exchange() {
        let currency_exchange: CurrencyExchange = init_testing_currency_exchange();

        let currency_to: Currency = Currency::EUR;
        let initial_data_frame: DataFrame = df!(
            "date" => [
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 3).unwrap(),
            ],
            "currency" => ["EUR", "CHF", "SEK"],
            "value" => [1.0, 1.0, 1.0]
        )
        .unwrap();

        let expected_data_frame: DataFrame = df!(
            "date" => [
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
                NaiveDate::from_ymd_opt(2020, 1, 3).unwrap(),
            ],
            "value" => [1.0, 1.0, 0.5]
        )
        .unwrap();

        let actual_data_frame: DataFrame = currency_exchange
            .exchange_currencies(&currency_to, initial_data_frame)
            .unwrap();

        assert!(expected_data_frame.equals(&actual_data_frame))
    }
}
