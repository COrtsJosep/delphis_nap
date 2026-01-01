#[cfg(test)]
mod tests {
    use crate::modules::database::DataBase;
    use crate::modules::financial::*;
    use crate::tests::test_helpers::init_party;
    use polars::df;

    #[test]
    fn correct_insert_party() {
        let mut data_base: DataBase = DataBase::new().unwrap();
        let mut party: Party = init_party();

        let _ = data_base.insert_party(&mut party);

        let expected_result = df!(
                "table" => ["income", "expenses", "funds", "party", "entity", "account"],
                "records" => [1, 1, 2, 1, 1, 1]
        )
        .unwrap();

        let actual_result = data_base.size().unwrap();

        assert!(actual_result.equals(&expected_result));
    }

    #[test]
    fn correct_insert_entity_account() {
        let mut data_base: DataBase = DataBase::new().unwrap();
        let entity = Entity::new(
            String::from("Aldi"),
            String::from("Germany"),
            EntityType::Firm,
            String::from("Supermarket"),
        );

        let account = Account::new(
            String::from("Current account"),
            String::from("Credit Suisse"),
            Currency::CHF,
            AccountType::Deposit,
            1080.0f64,
        );

        let _ = data_base.insert_entity(&entity);
        let _ = data_base.insert_account(&account);

        let expected_result = df!(
                "table" => ["income", "expenses", "funds", "party", "entity", "account"],
                "records" => [0, 0, 0, 0, 2, 2]
        )
        .unwrap();

        let actual_result = data_base.size().unwrap();

        assert!(actual_result.equals(&expected_result));
    }
}
