use crate::financial::*;
use crate::financial_database::DATE_FORMAT;
use crate::FinancialDataBase;
use chrono::{Local, NaiveDate};
use std::str::FromStr;
use std::vec::IntoIter;

impl FinancialDataBase {
    pub(crate) async fn insert_account(&mut self, account: &Account) -> Result<(), sqlx::Error> {
        let query_result = sqlx::query!("select max(account_id) as max_account_id from accounts")
            .fetch_one(&mut self.connection)
            .await;
        let account_id: i64 = match query_result {
            Ok(id) => id.max_account_id.unwrap() + 1i64,
            Err(_e) => 0i64,
        };

        let account_name: String = account.name();
        let account_country: String = account.country();
        let account_currency: String = account.currency().to_string();
        let account_type: String = account.account_type().to_string();
        let account_initial_balance: f64 = account.initial_balance();
        let account_creation_date: String =
            Local::now().date_naive().format(DATE_FORMAT).to_string();
        sqlx::query_file!(
            "src/queries/insertion/insert_into_accounts.sql",
            account_id,
            account_name,
            account_country,
            account_currency,
            account_type,
            account_initial_balance,
            account_creation_date,
        )
        .execute(&mut self.connection)
        .await?;

        Ok(())
    }

    pub(crate) async fn insert_entity(&mut self, entity: &Entity) -> Result<(), sqlx::Error> {
        let query_result = sqlx::query!("select max(entity_id) as max_entity_id from entities")
            .fetch_one(&mut self.connection)
            .await;
        let entity_id: i64 = match query_result {
            Ok(id) => id.max_entity_id.unwrap() + 1i64,
            Err(_e) => 0i64,
        };

        let entity_name: String = entity.name();
        let entity_country: String = entity.country();
        let entity_type: String = entity.entity_type().to_string();
        let entity_subtype: String = entity.entity_subtype();
        let entity_creation_date: String =
            Local::now().date_naive().format(DATE_FORMAT).to_string();
        sqlx::query_file!(
            "src/queries/insertion/insert_into_entities.sql",
            entity_id,
            entity_name,
            entity_country,
            entity_type,
            entity_subtype,
            entity_creation_date,
        )
        .execute(&mut self.connection)
        .await?;

        Ok(())
    }

    pub(crate) async fn insert_party(&mut self, party: &mut Party) -> Result<(), sqlx::Error> {
        let query_result = sqlx::query!("select max(party_id) as max_party_id from parties")
            .fetch_one(&mut self.connection)
            .await;
        let party_id: i64 = match query_result {
            Ok(id) => id.max_party_id.unwrap() + 1i64,
            Err(_e) => 0i64,
        };

        let party_creation_date: String = party.creation_date.format(DATE_FORMAT).to_string();
        sqlx::query_file!(
            "src/queries/insertion/insert_into_parties.sql",
            party_id,
            party_creation_date
        )
        .execute(&mut self.connection)
        .await?;

        for transaction in party.iter() {
            self.insert_transaction(&transaction, party_id).await?;
        }

        Ok(())
    }

    // sorry for the long method
    async fn insert_transaction(
        &mut self,
        transaction: &Transaction,
        party_id: i64,
    ) -> Result<(), sqlx::Error> {
        match transaction {
            Transaction::Expense {
                value,
                currency,
                date,
                category,
                subcategory,
                description,
                entity_id,
            } => {
                let query_result =
                    sqlx::query!("select max(expense_id) as max_expense_id from expenses")
                        .fetch_one(&mut self.connection)
                        .await;
                let expense_id: i64 = match query_result {
                    Ok(id) => id.max_expense_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let expense_date: String = date.format(DATE_FORMAT).to_string();
                let expense_currency: String = currency.to_string();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_expenses.sql",
                    expense_id,
                    value,
                    expense_currency,
                    expense_date,
                    category,
                    subcategory,
                    description,
                    entity_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
            Transaction::Income {
                value,
                currency,
                date,
                category,
                subcategory,
                description,
                entity_id,
            } => {
                let query_result =
                    sqlx::query!("select max(income_id) as max_income_id from incomes")
                        .fetch_one(&mut self.connection)
                        .await;
                let income_id: i64 = match query_result {
                    Ok(id) => id.max_income_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let income_date: String = date.format(DATE_FORMAT).to_string();
                let income_currency: String = currency.to_string();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_incomes.sql",
                    income_id,
                    value,
                    income_currency,
                    income_date,
                    category,
                    subcategory,
                    description,
                    entity_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
            Transaction::Credit {
                value,
                currency,
                date,
                account_id,
            } => {
                let query_result = sqlx::query!(
                    "select max(fund_movement_id) as max_fund_movement_id from fund_movements"
                )
                .fetch_one(&mut self.connection)
                .await;
                let fund_movement_id: i64 = match query_result {
                    Ok(id) => id.max_fund_movement_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let fund_movement_date: String = date.format(DATE_FORMAT).to_string();
                let fund_movement_currency: String = currency.to_string();
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_fund_movements.sql",
                    fund_movement_id,
                    "Credit",
                    value,
                    fund_movement_currency,
                    fund_movement_date,
                    account_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
            Transaction::Debit {
                value,
                currency,
                date,
                account_id,
            } => {
                let query_result = sqlx::query!(
                    "select max(fund_movement_id) as max_fund_movement_id from fund_movements"
                )
                .fetch_one(&mut self.connection)
                .await;
                let fund_movement_id: i64 = match query_result {
                    Ok(id) => id.max_fund_movement_id.unwrap() + 1i64,
                    Err(_e) => 0i64,
                };
                let fund_movement_date: String = date.format(DATE_FORMAT).to_string();
                let fund_movement_currency: String = currency.to_string();
                let fund_movement_value: f64 = -1.0 * value;
                sqlx::query_file!(
                    "src/queries/insertion/insert_into_fund_movements.sql",
                    fund_movement_id,
                    "Debit",
                    fund_movement_value,
                    fund_movement_currency,
                    fund_movement_date,
                    account_id,
                    party_id,
                )
                .execute(&mut self.connection)
                .await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn iter_entity_ids(&mut self) -> Result<IntoIter<i64>, sqlx::Error> {
        let rows = sqlx::query!("select entity_id from entities order by name asc limit 20")
            .fetch_all(&mut self.connection)
            .await?;

        let entity_ids: Vec<i64> = rows.into_iter().map(|r| r.entity_id).collect();
        Ok(entity_ids.into_iter())
    }

    pub(crate) async fn entity(&mut self, entity_id: i64) -> Result<Entity, sqlx::Error> {
        let row = sqlx::query!("select * from entities where entity_id = ?", entity_id)
            .fetch_one(&mut self.connection)
            .await?;

        let entity: Entity = Entity::new(
            row.name,
            row.country,
            EntityType::from_str(row.entity_type.as_str()).unwrap(),
            row.entity_subtype,
        );

        Ok(entity)
    }

    pub(crate) async fn entity_countries(&mut self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!("select distinct country from entities order by country asc")
            .fetch_all(&mut self.connection)
            .await?;

        let entity_countries = rows.into_iter().map(|r| r.country).collect();
        Ok(entity_countries)
    }

    pub(crate) async fn entity_subtypes(&mut self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!(
            "select distinct entity_subtype from entities order by entity_subtype asc"
        )
        .fetch_all(&mut self.connection)
        .await?;

        let entity_subtypes = rows.into_iter().map(|r| r.entity_subtype).collect();
        Ok(entity_subtypes)
    }

    pub(crate) async fn iter_account_ids(&mut self) -> Result<IntoIter<i64>, sqlx::Error> {
        let rows = sqlx::query!("select account_id from accounts order by name asc")
            .fetch_all(&mut self.connection)
            .await?;

        let account_ids: Vec<i64> = rows.into_iter().map(|r| r.account_id).collect();
        Ok(account_ids.into_iter())
    }

    pub(crate) async fn account(&mut self, account_id: i64) -> Result<Account, sqlx::Error> {
        let row = sqlx::query!("select * from accounts where account_id = ?", account_id)
            .fetch_one(&mut self.connection)
            .await?;

        let account: Account = Account::new(
            row.name,
            row.country,
            Currency::from_str(row.currency.as_str()).unwrap(),
            AccountType::from_str(row.account_type.as_str()).unwrap(),
            row.initial_balance,
        );

        Ok(account)
    }

    pub(crate) async fn account_countries(&mut self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query!("select distinct country from accounts order by country asc")
            .fetch_all(&mut self.connection)
            .await?;

        let account_countries = rows.into_iter().map(|r| r.country).collect();
        Ok(account_countries)
    }

    pub(crate) async fn transaction_categories(
        &mut self,
        transaction_type: &TransactionType,
    ) -> Result<Vec<String>, sqlx::Error> {
        let transaction_categories = match transaction_type {
            TransactionType::Income => {
                let rows =
                    sqlx::query!("select distinct category from incomes order by category asc")
                        .fetch_all(&mut self.connection)
                        .await?;
                rows.into_iter().map(|r| r.category).collect()
            }
            TransactionType::Expense => {
                let rows =
                    sqlx::query!("select distinct category from expenses order by category asc")
                        .fetch_all(&mut self.connection)
                        .await?;
                rows.into_iter().map(|r| r.category).collect()
            }
            _ => Vec::new(), // Should never happen for Credit/Debit
        };

        Ok(transaction_categories)
    }

    pub(crate) async fn transaction_subcategories(
        &mut self,
        transaction_type: &TransactionType,
        category: String,
    ) -> Result<Vec<String>, sqlx::Error> {
        let transaction_subcategories = match transaction_type {
            TransactionType::Income => {
                let rows = sqlx::query!(
                        "select distinct subcategory from incomes where category = ? order by subcategory asc",
                        category
                    )
                    .fetch_all(&mut self.connection)
                    .await?;
                rows.into_iter().map(|r| r.subcategory).collect()
            }
            TransactionType::Expense => {
                let rows = sqlx::query!(
                        "select distinct subcategory from expenses where category = ? order by subcategory asc",
                        category
                    )
                    .fetch_all(&mut self.connection)
                    .await?;
                rows.into_iter().map(|r| r.subcategory).collect()
            }
            _ => Vec::new(), // Should never happen for Credit/Debit
        };

        Ok(transaction_subcategories)
    }

    async fn transaction(
        &mut self,
        transaction_type: TransactionType,
        transaction_id: i64,
    ) -> Result<Transaction, sqlx::Error> {
        let transaction = match transaction_type {
            TransactionType::Expense => {
                let row = sqlx::query!(
                    "select * from expenses where expense_id = ?",
                    transaction_id
                )
                .fetch_one(&mut self.connection)
                .await?;

                Transaction::Expense {
                    value: row.value,
                    currency: Currency::from_str(row.currency.as_str()).unwrap(),
                    date: NaiveDate::parse_from_str(row.date.as_str(), DATE_FORMAT).unwrap(),
                    category: row.category,
                    subcategory: row.subcategory,
                    description: row.description,
                    entity_id: row.entity_id,
                }
            }
            TransactionType::Income => {
                let row = sqlx::query!("select * from incomes where income_id = ?", transaction_id)
                    .fetch_one(&mut self.connection)
                    .await?;

                Transaction::Income {
                    value: row.value,
                    currency: Currency::from_str(row.currency.as_str()).unwrap(),
                    date: NaiveDate::parse_from_str(row.date.as_str(), DATE_FORMAT).unwrap(),
                    category: row.category,
                    subcategory: row.subcategory,
                    description: row.description,
                    entity_id: row.entity_id,
                }
            }
            TransactionType::Credit => {
                let row = sqlx::query!(
                    "select * from fund_movements where fund_movement_id = ?",
                    transaction_id
                )
                .fetch_one(&mut self.connection)
                .await?;

                Transaction::Credit {
                    value: row.value,
                    currency: Currency::from_str(row.currency.as_str()).unwrap(),
                    date: NaiveDate::parse_from_str(row.date.as_str(), DATE_FORMAT).unwrap(),
                    account_id: row.account_id,
                }
            }
            TransactionType::Debit => {
                let row = sqlx::query!(
                    "select * from fund_movements where fund_movement_id = ?",
                    transaction_id
                )
                .fetch_one(&mut self.connection)
                .await?;

                Transaction::Debit {
                    value: -1.0 * row.value,
                    currency: Currency::from_str(row.currency.as_str()).unwrap(),
                    date: NaiveDate::parse_from_str(row.date.as_str(), DATE_FORMAT).unwrap(),
                    account_id: row.account_id,
                }
            }
        };

        Ok(transaction)
    }

    pub(crate) async fn party(&mut self, party_id: i64) -> Result<Party, sqlx::Error> {
        let mut party: Party = Party::new(Vec::new());

        // add incomes to the party
        let query_result =
            sqlx::query!("select income_id from incomes where party_id = ?", party_id)
                .fetch_all(&mut self.connection)
                .await?;

        let income_ids: Vec<i64> = query_result.into_iter().map(|r| r.income_id).collect();
        for income_id in income_ids {
            party.add_transaction(self.transaction(TransactionType::Income, income_id).await?);
        }

        // add expenses to the party
        let query_result = sqlx::query!(
            "select expense_id from expenses where party_id = ?",
            party_id
        )
        .fetch_all(&mut self.connection)
        .await?;

        let expense_ids: Vec<i64> = query_result.into_iter().map(|r| r.expense_id).collect();
        for expense_id in expense_ids {
            party.add_transaction(
                self.transaction(TransactionType::Expense, expense_id)
                    .await?,
            );
        }

        // add fund movements to the party
        let query_result = sqlx::query!(
            "select fund_movement_id, fund_movement_type from fund_movements where party_id = ?",
            party_id
        )
        .fetch_all(&mut self.connection)
        .await?;

        for record in query_result {
            let transaction_type = match record.fund_movement_type == "Credit" {
                true => TransactionType::Credit,
                false => TransactionType::Debit,
            };
            party.add_transaction(
                self.transaction(transaction_type, record.fund_movement_id)
                    .await?,
            );
        }

        Ok(party)
    }

    pub(crate) async fn delete_party(&mut self, party_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("delete from expenses where party_id = ?", party_id)
            .execute(&mut self.connection)
            .await?;
        sqlx::query!("delete from incomes where party_id = ?", party_id)
            .execute(&mut self.connection)
            .await?;
        sqlx::query!("delete from fund_movements where party_id = ?", party_id)
            .execute(&mut self.connection)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table_records::*;
    use crate::{TEST_FINANCIAL_DATABASE_URL, TEST_ORIGINAL_FINANCIAL_DATABASE_URL};
    use sqlx::{migrate::MigrateDatabase, Sqlite};

    /// tears down the test database and sets it up again
    async fn reset_database() -> () {
        if Sqlite::database_exists(TEST_FINANCIAL_DATABASE_URL)
            .await
            .unwrap()
        {
            Sqlite::drop_database(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();
        }

        std::fs::copy(
            TEST_ORIGINAL_FINANCIAL_DATABASE_URL
                .strip_prefix("sqlite://")
                .unwrap(),
            TEST_FINANCIAL_DATABASE_URL
                .strip_prefix("sqlite://")
                .unwrap(),
        )
        .unwrap();
    }

    #[tokio::test]
    async fn test_insert_account() {
        reset_database().await;

        let mut financial_database: FinancialDataBase =
            FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();

        let account = Account::new(
            String::from("Credit Suisse"),
            String::from("Switzerland"),
            Currency::CHF,
            AccountType::Deposit,
            1080.0f64,
        );

        let expected_account_record: AccountRecord = AccountRecord {
            account_id: 1i64,
            name: String::from("Credit Suisse"),
            country: String::from("Switzerland"),
            currency: String::from("CHF"),
            account_type: String::from("Deposit"),
            initial_balance: 1080.0f64,
            creation_date: Local::now().date_naive().format(DATE_FORMAT).to_string(),
        };

        financial_database.insert_account(&account).await.unwrap();

        let actual_account_record: AccountRecord =
            sqlx::query_as!(AccountRecord, "select * from accounts where account_id = 1")
                .fetch_one(&mut financial_database.connection)
                .await
                .unwrap();

        assert_eq!(expected_account_record, actual_account_record)
    }

    #[tokio::test]
    async fn test_insert_entity() {
        reset_database().await;

        let mut financial_database: FinancialDataBase =
            FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();

        let entity = Entity::new(
            String::from("Aldi"),
            String::from("Germany"),
            EntityType::Firm,
            String::from("Supermarket"),
        );

        let expected_entity_record: EntityRecord = EntityRecord {
            entity_id: 1i64,
            name: String::from("Aldi"),
            country: String::from("Germany"),
            entity_type: String::from("Firm"),
            entity_subtype: String::from("Supermarket"),
            creation_date: Local::now().date_naive().format(DATE_FORMAT).to_string(),
        };

        financial_database.insert_entity(&entity).await.unwrap();

        let actual_entity_record: EntityRecord =
            sqlx::query_as!(EntityRecord, "select * from entities where entity_id = 1")
                .fetch_one(&mut financial_database.connection)
                .await
                .unwrap();

        assert_eq!(expected_entity_record, actual_entity_record)
    }

    #[tokio::test]
    async fn test_insert_expense() {
        reset_database().await;

        let mut financial_database: FinancialDataBase =
            FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();

        let expense = Transaction::Expense {
            value: 100.0,
            currency: Currency::EUR,
            date: Local::now().date_naive(),
            category: String::from("Groceries"),
            subcategory: String::from("Food"),
            description: String::from("Weekly shopping"),
            entity_id: 0,
        };

        let expected_expense_record: ExpenseRecord = ExpenseRecord {
            expense_id: 262i64,
            value: 100.0f64,
            currency: String::from("EUR"),
            date: Local::now().date_naive().format(DATE_FORMAT).to_string(),
            category: String::from("Groceries"),
            subcategory: String::from("Food"),
            description: String::from("Weekly shopping"),
            entity_id: 0i64,
            party_id: 0i64,
        };

        financial_database
            .insert_transaction(&expense, 0i64)
            .await
            .unwrap();

        let actual_expense_record: ExpenseRecord = sqlx::query_as!(
            ExpenseRecord,
            "select * from expenses where expense_id = 262"
        )
        .fetch_one(&mut financial_database.connection)
        .await
        .unwrap();

        assert_eq!(expected_expense_record, actual_expense_record);
    }

    #[tokio::test]
    async fn test_insert_income() {
        reset_database().await;

        let mut financial_database: FinancialDataBase =
            FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();

        let income = Transaction::Income {
            value: 5000.0,
            currency: Currency::CHF,
            date: Local::now().date_naive(),
            category: String::from("Salary"),
            subcategory: String::from("Monthly"),
            description: String::from("March salary"),
            entity_id: 0,
        };

        let expected_income_record: IncomeRecord = IncomeRecord {
            income_id: 12i64,
            value: 5000.0f64,
            currency: String::from("CHF"),
            date: Local::now().date_naive().format(DATE_FORMAT).to_string(),
            category: String::from("Salary"),
            subcategory: String::from("Monthly"),
            description: String::from("March salary"),
            entity_id: 0i64,
            party_id: 0i64,
        };

        financial_database
            .insert_transaction(&income, 0i64)
            .await
            .unwrap();

        let actual_income_record: IncomeRecord =
            sqlx::query_as!(IncomeRecord, "select * from incomes where income_id = 12")
                .fetch_one(&mut financial_database.connection)
                .await
                .unwrap();

        assert_eq!(expected_income_record, actual_income_record);
    }

    #[tokio::test]
    async fn test_insert_credit() {
        reset_database().await;

        let mut financial_database: FinancialDataBase =
            FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();

        let credit = Transaction::Credit {
            value: 200.0,
            currency: Currency::CHF,
            date: Local::now().date_naive(),
            account_id: 0,
        };

        let expected_credit_record: FundMovementRecord = FundMovementRecord {
            fund_movement_id: 274i64,
            fund_movement_type: String::from("Credit"),
            value: 200.0f64,
            currency: String::from("CHF"),
            date: Local::now().date_naive().format(DATE_FORMAT).to_string(),
            account_id: 0i64,
            party_id: 0i64,
        };

        financial_database
            .insert_transaction(&credit, 0i64)
            .await
            .unwrap();

        let actual_credit_record: FundMovementRecord = sqlx::query_as!(
            FundMovementRecord,
            "select * from fund_movements where fund_movement_id = 274"
        )
        .fetch_one(&mut financial_database.connection)
        .await
        .unwrap();

        assert_eq!(expected_credit_record, actual_credit_record);
    }

    #[tokio::test]
    async fn test_insert_debit() {
        reset_database().await;

        let mut financial_database: FinancialDataBase =
            FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();

        let debit = Transaction::Debit {
            value: 100.0,
            currency: Currency::CHF,
            date: Local::now().date_naive(),
            account_id: 0,
        };

        let expected_debit_record: FundMovementRecord = FundMovementRecord {
            fund_movement_id: 274i64,
            fund_movement_type: String::from("Debit"),
            value: -100.0f64, // Note: Debit is stored as negative
            currency: String::from("CHF"),
            date: Local::now().date_naive().format(DATE_FORMAT).to_string(),
            account_id: 0i64,
            party_id: 0i64,
        };

        financial_database
            .insert_transaction(&debit, 0i64)
            .await
            .unwrap();

        let actual_debit_record: FundMovementRecord = sqlx::query_as!(
            FundMovementRecord,
            "select * from fund_movements where fund_movement_id = 274"
        )
        .fetch_one(&mut financial_database.connection)
        .await
        .unwrap();

        assert_eq!(expected_debit_record, actual_debit_record);
    }

    #[tokio::test]
    async fn test_insert_party() {
        reset_database().await;

        let mut financial_database: FinancialDataBase =
            FinancialDataBase::init(TEST_FINANCIAL_DATABASE_URL)
                .await
                .unwrap();

        let mut party: Party = Party {
            transactions: vec![],
            creation_date: Local::now().date_naive(),
        };

        let expected_party_record: PartyRecord = PartyRecord {
            party_id: 274i64,
            creation_date: Local::now().date_naive().format(DATE_FORMAT).to_string(),
        };

        financial_database.insert_party(&mut party).await.unwrap();

        let actual_party_record: PartyRecord =
            sqlx::query_as!(PartyRecord, "select * from parties where party_id = 274")
                .fetch_one(&mut financial_database.connection)
                .await
                .unwrap();

        assert_eq!(expected_party_record, actual_party_record);
    }
}
