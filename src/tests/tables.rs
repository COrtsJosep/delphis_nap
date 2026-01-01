#[cfg(test)]
mod tests {
    use crate::modules::financial::*;
    use crate::modules::tables::*;
    use chrono::prelude::*;
    use polars::prelude::*;

    fn init_funds_table() -> FundsTable {
        let data_frame: DataFrame = df!(
            "fund_movement_id" => [0i64, 1i64],
            "fund_movement_type" => ["Credit", "Debit"],
            "value" => [1309.23f64, -89.0f64],
            "currency" => [Currency::EUR.to_string(), Currency::EUR.to_string()],
            "date" => [
                NaiveDate::from_ymd_opt(1997, 1, 10).unwrap(),
                NaiveDate::from_ymd_opt(1985, 2, 15).unwrap()
            ],
            "account_id" => [0i64, 0i64],
            "party_id" => [0i64, 1i64],
        )
        .unwrap();

        FundsTable { data_frame }
    }

    #[test]
    fn correct_funds_table_init() {
        let funds_table: Box<FundsTable> = FundsTable::new().unwrap();

        assert!(funds_table.data_frame.is_empty());
    }

    #[test]
    fn correct_id_empty_funds_table_init() {
        let mut funds_table: FundsTable = *FundsTable::new().unwrap();

        let transaction = Transaction::Debit {
            value: 300.0,
            currency: Currency::EUR,
            date: NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
            account_id: 0i64,
        };

        let _ = funds_table.insert_transaction(&transaction, 0);

        let binding = funds_table
            .data_frame()
            .column(format!("{}_id", FundsTable::name()).as_str())
            .unwrap()
            .max_reduce()
            .unwrap();
        let actual_last_id = binding.value();
        let expected_last_id = AnyValue::Int64(0i64);

        assert_eq!(actual_last_id, &expected_last_id)
    }

    #[test]
    fn correct_id_nonempty_funds_table_insertion() {
        let mut funds_table: FundsTable = init_funds_table();
        let transaction = Transaction::Debit {
            value: 300.0,
            currency: Currency::EUR,
            date: NaiveDate::from_ymd_opt(2024, 12, 2).unwrap(),
            account_id: 0i64,
        };

        let _ = funds_table.insert_transaction(&transaction, 0);

        let binding = funds_table
            .data_frame()
            .column(format!("{}_id", FundsTable::name()).as_str())
            .unwrap()
            .max_reduce()
            .unwrap();
        let actual_last_id = binding.value();
        let expected_last_id = AnyValue::Int64(2i64);

        assert_eq!(actual_last_id, &expected_last_id)
    }

    #[test]
    fn correct_entity_table_init() {
        let entity_table: EntityTable = *EntityTable::new().unwrap();

        assert_eq!(entity_table.data_frame.height(), 1);
    }

    #[test]
    fn correct_id_empty_entity_table_init() {
        let mut entity_table: EntityTable = *EntityTable::new().unwrap();

        let entity = Entity::new(
            String::from("Aldi"),
            String::from("Germany"),
            EntityType::Firm,
            String::from("Supermarket"),
        );

        let _ = entity_table.insert_entity(&entity);

        let binding = entity_table
            .data_frame()
            .column(format!("{}_id", EntityTable::name()).as_str())
            .unwrap()
            .max_reduce()
            .unwrap();
        let actual_last_id = binding.value();
        let expected_last_id = AnyValue::Int64(1i64);

        assert_eq!(actual_last_id, &expected_last_id)
    }

    #[test]
    fn correct_id_empty_account_table_init() {
        let mut account_table: AccountTable = *AccountTable::new().unwrap();

        let account = Account::new(
            String::from("Current account"),
            String::from("Credit Suisse"),
            Currency::CHF,
            AccountType::Deposit,
            1080.0f64,
        );

        let _ = account_table.insert_account(&account);

        let binding = account_table
            .data_frame()
            .column(format!("{}_id", AccountTable::name()).as_str())
            .unwrap()
            .max_reduce()
            .unwrap();
        let actual_last_id = binding.value();
        let expected_last_id = AnyValue::Int64(1i64);

        assert_eq!(actual_last_id, &expected_last_id)
    }

    #[test]
    fn correct_income_table_delete() {
        let mut income_table = *IncomeTable::init().unwrap();
        let orig_size = income_table.data_frame.size();
        let party_0_size = income_table
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").eq(lit(0)))
            .collect()
            .unwrap()
            .size();
        let _ = income_table.delete_party(0);

        assert_eq!(orig_size - party_0_size, income_table.data_frame.size())
    }

    #[test]
    fn correct_income_table_transaction() {
        let mut income_table = *IncomeTable::new().unwrap();
        let original_transaction = Transaction::Income {
            value: 0.0,
            currency: Currency::EUR,
            date: NaiveDate::default(),
            category: String::from("Test category"),
            subcategory: String::from("Test subcategory"),
            description: String::from("Test description"),
            entity_id: 0,
        };
        let _ = income_table.insert_transaction(&original_transaction, 0);
        let returned_transaction = income_table.transaction(0).unwrap();

        assert!(
            (original_transaction.value() == returned_transaction.value())
                & (original_transaction.currency() == returned_transaction.currency())
                & (original_transaction.date() == returned_transaction.date())
        );
    }
}
