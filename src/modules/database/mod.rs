mod palettes;
pub mod plotter;
pub mod summaries;
pub mod views;

use crate::modules::financial::*;
use crate::modules::tables::*;
use polars::prelude::*;
use regex::Regex;
use std::io::Cursor;
use std::vec::IntoIter;

fn data_frame_to_csv_string(
    data_frame: &mut DataFrame,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = Cursor::new(Vec::new());

    CsvWriter::new(&mut buffer)
        .include_header(true)
        .finish(data_frame)?;

    let re = Regex::new(r"(\.\d)([\,\n])")?;

    Ok(re
        .replace_all(
            String::from_utf8(buffer.into_inner()).unwrap().as_str(),
            "${1}0${2}",
        )
        .to_string()
        .trim_end_matches("\n")
        .to_string())
}

fn capitalize_every_word(sentence: String) -> String {
    // Copied and addapted to my needs from thirtyseconds
    // https://docs.rs/thirtyseconds/latest/thirtyseconds/strings/fn.capitalize_every_word.html
    sentence
        .as_str()
        .split(' ')
        .map(|word| format!("{}{}", &word[..1].to_uppercase(), &word[1..]))
        .collect::<Vec<_>>()
        .join(" ")
}

pub struct DataBase {
    incomes_table: IncomeTable,
    expenses_table: ExpensesTable,
    funds_table: FundsTable,
    party_table: PartyTable,
    entity_table: EntityTable,
    account_table: AccountTable,
}

impl DataBase {
    pub(crate) fn new() -> Result<DataBase, PolarsError> {
        let incomes_table = *IncomeTable::new()?;
        let expenses_table = *ExpensesTable::new()?;
        let funds_table = *FundsTable::new()?;
        let party_table = *PartyTable::new()?;
        let entity_table = *EntityTable::new()?;
        let account_table = *AccountTable::new()?;

        Ok(DataBase {
            incomes_table,
            expenses_table,
            funds_table,
            party_table,
            entity_table,
            account_table,
        })
    }

    pub fn init() -> Result<DataBase, PolarsError> {
        let incomes_table = *IncomeTable::init()?;
        let expenses_table = *ExpensesTable::init()?;
        let funds_table = *FundsTable::init()?;
        let party_table = *PartyTable::init()?;
        let entity_table = *EntityTable::init()?;
        let account_table = *AccountTable::init()?;

        Ok(DataBase {
            incomes_table,
            expenses_table,
            funds_table,
            party_table,
            entity_table,
            account_table,
        })
    }

    pub fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.incomes_table.save()?;
        self.expenses_table.save()?;
        self.funds_table.save()?;
        self.party_table.save()?;
        self.entity_table.save()?;
        self.account_table.save()?;

        Ok(())
    }

    pub fn insert_party(&mut self, party: &mut Party) -> Result<(), Box<dyn std::error::Error>> {
        let party_id: i64 = self.party_table.next_id()?;
        for transaction in party.iter() {
            self.insert_transaction(&transaction, party_id)?;
        }

        self.party_table.insert_party(party)?;

        Ok(())
    }

    fn insert_transaction(
        &mut self,
        transaction: &Transaction,
        party_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match transaction {
            Transaction::Expense { .. } => self
                .expenses_table
                .insert_transaction(transaction, party_id)?,
            Transaction::Income { .. } => {
                self.incomes_table
                    .insert_transaction(transaction, party_id)?;
            }
            Transaction::Credit { .. } | Transaction::Debit { .. } => {
                self.funds_table.insert_transaction(transaction, party_id)?;
            }
        }

        Ok(())
    }

    /// Returns the number of records in each table, for testing purposes
    pub(crate) fn size(&self) -> Result<DataFrame, PolarsError> {
        let data_frame: DataFrame = df!(
            "table" => ["income", "expenses", "funds", "party", "entity", "account"],
            "records" => [
                self.incomes_table.data_frame.height() as i64,
                self.expenses_table.data_frame.height() as i64,
                self.funds_table.data_frame.height() as i64,
                self.party_table.data_frame.height() as i64,
                self.entity_table.data_frame.height() as i64,
                self.account_table.data_frame.height() as i64
            ]
        )?;

        Ok(data_frame)
    }

    pub fn insert_entity(&mut self, entity: &Entity) -> Result<i64, PolarsError> {
        self.entity_table.insert_entity(entity)
    }

    pub fn insert_account(&mut self, account: &Account) -> Result<i64, PolarsError> {
        self.account_table.insert_account(account)
    }

    pub(crate) fn iter_entity_ids(&mut self) -> Result<IntoIter<i64>, PolarsError> {
        self.entity_table.iter()
    }

    pub(crate) fn entity(&self, entity_id: i64) -> Result<Entity, Box<dyn std::error::Error>> {
        self.entity_table.entity(entity_id)
    }

    pub(crate) fn iter_account_ids(&mut self) -> Result<IntoIter<i64>, PolarsError> {
        self.account_table.iter()
    }

    pub(crate) fn entity_countries(&self) -> Result<Vec<String>, PolarsError> {
        self.entity_table.countries()
    }

    pub(crate) fn account(&self, account_id: i64) -> Result<Account, Box<dyn std::error::Error>> {
        self.account_table.account(account_id)
    }

    pub(crate) fn account_countries(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.account_table.countries()
    }

    pub(crate) fn transaction_categories(
        &self,
        transaction_type: &TransactionType,
    ) -> Result<Vec<String>, PolarsError> {
        match transaction_type {
            TransactionType::Income => self.incomes_table.categories(),
            TransactionType::Expense => self.expenses_table.categories(),
            _ => Ok(Vec::new()), // rethink whether it's the correct thing to do
        }
    }

    pub(crate) fn transaction_subcategories(
        &self,
        transaction_type: &TransactionType,
        category: String,
    ) -> Result<Vec<String>, PolarsError> {
        match transaction_type {
            TransactionType::Income => self.incomes_table.subcategories(category),
            TransactionType::Expense => self.expenses_table.subcategories(category),
            _ => Ok(Vec::new()),
        }
    }

    pub(crate) fn entity_subtypes(&self) -> Result<Vec<String>, PolarsError> {
        self.entity_table.subtypes()
    }

    /// Deletes from the database all records from the party.
    pub(crate) fn delete_party(&mut self, party_id: i64) -> Result<(), PolarsError> {
        self.incomes_table.delete_party(party_id)?;
        self.expenses_table.delete_party(party_id)?;
        self.funds_table.delete_party(party_id)?;
        self.party_table.delete_party(party_id)?;

        Ok(())
    }

    pub(crate) fn party(&self, party_id: i64) -> Result<Party, Box<dyn std::error::Error>> {
        let mut party: Party = Party::new(Vec::new());
        for income_id in self.incomes_table.iter_party(party_id)? {
            party.add_transaction(self.incomes_table.transaction(income_id)?);
        }
        for expense_id in self.expenses_table.iter_party(party_id)? {
            party.add_transaction(self.expenses_table.transaction(expense_id)?);
        }
        for fund_id in self.funds_table.iter_party(party_id)? {
            party.add_transaction(self.funds_table.transaction(fund_id)?);
        }

        Ok(party)
    }
}

impl Default for DataBase {
    fn default() -> Self {
        // think how to handle panic
        Self::init().expect("Failed to initialize database!!!")
    }
}
