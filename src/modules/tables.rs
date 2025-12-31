use super::financial::{Account, AccountType, Currency, Entity, EntityType, Party, Transaction};
use chrono::{Local, NaiveDate};
use polars::prelude::*;
use std::fs::{create_dir, File};
use std::path::Path;
use std::str::FromStr;
use std::vec::IntoIter;
use std::{error::Error, fmt};

#[derive(Debug, Clone)]
struct IncorrectTableError;
impl Error for IncorrectTableError {}
impl fmt::Display for IncorrectTableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attempted to insert transaction into the wrong table!")
    }
}

pub trait Table {
    /// Returns the name of the table
    fn name() -> String;

    /// Returns a reference to the dataframe of the table struct
    fn data_frame(&self) -> &DataFrame;

    /// Returns a mutable reference to the dataframe of the table struct
    fn mut_data_frame(&mut self) -> &mut DataFrame;

    /// Creates a table instance by consuming a dataframe
    fn create(data_frame: DataFrame) -> Box<Self>;

    /// Creates a table instance with zero rows
    fn new() -> Result<Box<Self>, PolarsError>;

    /// Creates a table instance by trying to load a csv in the right location
    fn try_load() -> Result<Box<Self>, PolarsError> {
        let data_frame = CsvReadOptions::default()
            .with_infer_schema_length(None)
            .with_has_header(true)
            .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
            .try_into_reader_with_file_path(Some(
                format!("data/{}_table.csv", Self::name()).into(),
            ))?
            .finish()?;

        Ok(Self::create(data_frame))
    }

    /// Creates a table instance by trying to load the csv data and,
    /// if there is none, by creating an empty one
    fn init() -> Result<Box<Self>, PolarsError> {
        match Self::try_load() {
            // tries to load
            Ok(s) => Ok(s), // if load -> ok
            Err(_) => match Self::new() {
                // if not -> try to create new
                Ok(n) => Ok(n),   // if new -> ok
                Err(e) => Err(e), // else -> return error
            },
        }
    }

    /// Saves the table data in the right location
    fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.data_frame().is_empty() {
            return Ok(());
        }

        let file_name: String = format!("data/{}_table.csv", Self::name());
        let path: &Path = Path::new(&file_name);
        let parent: &Path = path.parent().ok_or("Path does not have parent!")?;
        if !parent.exists() {
            let _ = create_dir(parent);
        }

        let mut file = File::create(path)?;

        CsvWriter::new(&mut file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut self.mut_data_frame())?;

        Ok(())
    }

    /// Gets the ID of the last record of the table + 1. If the table is empty,
    /// returns 0
    fn next_id(&self) -> Result<i64, PolarsError> {
        if self.data_frame().is_empty() {
            Ok(0i64)
        } else {
            if let AnyValue::Int64(id) = self
                .data_frame()
                .column(format!("{}_id", Self::name()).as_str())?
                .max_reduce()?
                .value()
            {
                Ok(id + 1i64)
            } else {
                Err(PolarsError::NoData(
                    format!("Failed to find last {}_id", Self::name()).into(),
                ))
            }
        }
    }

    /// Prints the table
    fn display(&self) {
        println!("{}", self.data_frame());
    }
}

pub struct IncomeTable {
    pub data_frame: DataFrame,
}

impl Table for IncomeTable {
    fn name() -> String {
        String::from("income")
    }

    fn data_frame(&self) -> &DataFrame {
        &self.data_frame
    }

    fn mut_data_frame(&mut self) -> &mut DataFrame {
        &mut self.data_frame
    }

    fn create(data_frame: DataFrame) -> Box<Self> {
        Box::new(IncomeTable { data_frame })
    }

    fn new() -> Result<Box<Self>, PolarsError> {
        let data_frame = DataFrame::new(vec![
            Column::from(Series::new(
                PlSmallStr::from(format!("{}_id", IncomeTable::name())),
                Vec::<i64>::new(),
            )),
            Column::from(Series::new(PlSmallStr::from("value"), Vec::<f64>::new())),
            Column::from(Series::new(
                PlSmallStr::from("currency"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("date"),
                Vec::<NaiveDate>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("category"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("subcategory"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("description"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("entity_id"),
                Vec::<i64>::new(),
            )),
            Column::from(Series::new(PlSmallStr::from("party_id"), Vec::<i64>::new())),
        ])?;

        Ok(IncomeTable::create(data_frame))
    }
}

impl IncomeTable {
    /// Adds income transaction to the table
    pub fn insert_transaction(
        &mut self,
        transaction: &Transaction,
        party_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Transaction::Income {
            value,
            currency,
            date,
            category,
            subcategory,
            description,
            entity_id,
        } = transaction
        {
            let id: i64 = self.next_id()?;

            let record = df!(
                    format!("{}_id", IncomeTable::name()) => [id],
                    "value" => [*value],
                    "currency" => [currency.to_string()],
                    "date" => [*date],
                    "category" => [category.to_string()],
                    "subcategory" => [subcategory.to_string()],
                    "description" => [description.to_string()],
                    "entity_id" => [*entity_id],
                    "party_id" => [party_id]
            )?;

            self.data_frame = self.data_frame().vstack(&record)?;

            Ok(())
        } else {
            Err(IncorrectTableError.into())
        }
    }

    pub(crate) fn categories(&self) -> Result<Vec<String>, PolarsError> {
        Ok(self
            .data_frame()
            .column("category")?
            .unique()?
            .str()?
            .sort(false)
            .into_no_null_iter()
            .map(|s| s.to_string())
            .collect())
    }

    pub(crate) fn subcategories(&self, category: String) -> Result<Vec<String>, PolarsError> {
        let mask = self
            .data_frame()
            .column("category")?
            .str()?
            .equal(category.as_str());

        Ok(self
            .data_frame()
            .filter(&mask)?
            .column("subcategory")?
            .unique()?
            .str()?
            .sort(false)
            .into_no_null_iter()
            .map(|s| s.to_string())
            .collect())
    }

    /// Deletes records corresponding to a party.
    pub(crate) fn delete_party(&mut self, party_id: i64) -> Result<(), PolarsError> {
        self.data_frame = self
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").neq(lit(party_id)))
            .collect()?;

        Ok(())
    }

    /// Returns iterator of income_ids that correspond to the given party_id
    pub(crate) fn iter_party(&self, party_id: i64) -> Result<IntoIter<i64>, PolarsError> {
        Ok(self
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").eq(lit(party_id)))
            .collect()?
            .column(format!("{}_id", IncomeTable::name()).as_str())?
            .i64()?
            .into_no_null_iter()
            .collect::<Vec<i64>>()
            .into_iter())
    }

    /// Returns entity given ID
    pub(crate) fn transaction(&self, id: i64) -> Result<Transaction, Box<dyn std::error::Error>> {
        let mask = self
            .data_frame
            .column(format!("{}_id", IncomeTable::name()).as_str())?
            .i64()?
            .equal(id);

        let record = self.data_frame.filter(&mask)?.clone();
        let date: NaiveDate = record
            .column("date")?
            .date()?
            .as_date_iter()
            .next()
            .ok_or("Could not find date!")?
            .clone()
            .ok_or("Could not find date!")?;

        let transaction: Transaction = Transaction::Income {
            value: record
                .column("value")?
                .f64()?
                .get(0)
                .ok_or("Could not find value!")?,
            currency: Currency::from_str(
                record
                    .column("currency")?
                    .str()?
                    .get(0)
                    .ok_or("Could not find currency!")?,
            )?,
            date,
            category: record
                .column("category")?
                .str()?
                .get(0)
                .ok_or("Could not find category!")?
                .to_string(),
            subcategory: record
                .column("subcategory")?
                .str()?
                .get(0)
                .ok_or("Could not find subcategory!")?
                .to_string(),
            description: record
                .column("description")?
                .str()?
                .get(0)
                .ok_or("Could not find description!")?
                .to_string(),
            entity_id: record
                .column("entity_id")?
                .i64()?
                .get(0)
                .ok_or("Could not find entity_id!")?,
        };

        Ok(transaction)
    }
}

pub struct ExpensesTable {
    pub data_frame: DataFrame,
}

impl Table for ExpensesTable {
    fn name() -> String {
        String::from("expense")
    }

    fn data_frame(&self) -> &DataFrame {
        &self.data_frame
    }

    fn mut_data_frame(&mut self) -> &mut DataFrame {
        &mut self.data_frame
    }

    fn create(data_frame: DataFrame) -> Box<Self> {
        Box::new(ExpensesTable { data_frame })
    }

    fn new() -> Result<Box<Self>, PolarsError> {
        let data_frame = DataFrame::new(vec![
            Column::from(Series::new(
                PlSmallStr::from("expense_id"),
                Vec::<i64>::new(),
            )),
            Column::from(Series::new(PlSmallStr::from("value"), Vec::<f64>::new())),
            Column::from(Series::new(
                PlSmallStr::from("currency"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("date"),
                Vec::<NaiveDate>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("category"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("subcategory"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("description"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("entity_id"),
                Vec::<i64>::new(),
            )),
            Column::from(Series::new(PlSmallStr::from("party_id"), Vec::<i64>::new())),
        ])?;

        Ok(ExpensesTable::create(data_frame))
    }
}

impl ExpensesTable {
    /// Adds expense transaction to the table
    pub fn insert_transaction(
        &mut self,
        transaction: &Transaction,
        party_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Transaction::Expense {
            value,
            currency,
            date,
            category,
            subcategory,
            description,
            entity_id,
        } = transaction
        {
            let id: i64 = self.next_id()?;

            let record = df!(
                format!("{}_id", ExpensesTable::name()) => [id],
                "value" => [*value],
                "currency" => [currency.to_string()],
                "date" => [*date],
                "category" => [category.to_string()],
                "subcategory" => [subcategory.to_string()],
                "description" => [description.to_string()],
                "entity_id" => [*entity_id],
                "party_id" => [party_id]
            )?;

            self.data_frame = self.data_frame.vstack(&record)?;

            Ok(())
        } else {
            Err(IncorrectTableError.into())
        }
    }

    pub(crate) fn categories(&self) -> Result<Vec<String>, PolarsError> {
        Ok(self
            .data_frame()
            .column("category")?
            .unique()?
            .str()?
            .sort(false)
            .into_no_null_iter()
            .map(|s| s.to_string())
            .collect())
    }

    pub(crate) fn subcategories(&self, category: String) -> Result<Vec<String>, PolarsError> {
        let mask = self
            .data_frame()
            .column("category")?
            .str()?
            .equal(category.as_str());

        Ok(self
            .data_frame()
            .filter(&mask)?
            .column("subcategory")?
            .unique()?
            .str()?
            .sort(false)
            .into_no_null_iter()
            .map(|s| s.to_string())
            .collect())
    }

    // Deletes records corresponding to a party.
    pub(crate) fn delete_party(&mut self, party_id: i64) -> Result<(), PolarsError> {
        self.data_frame = self
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").neq(lit(party_id)))
            .collect()?;

        Ok(())
    }

    /// Returns iterator of expenses_ids that correspond to the given party_id
    pub(crate) fn iter_party(&self, party_id: i64) -> Result<IntoIter<i64>, PolarsError> {
        Ok(self
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").eq(lit(party_id)))
            .collect()?
            .column(format!("{}_id", ExpensesTable::name()).as_str())?
            .i64()?
            .into_no_null_iter()
            .collect::<Vec<i64>>()
            .into_iter())
    }

    /// Returns entity given ID
    pub(crate) fn transaction(&self, id: i64) -> Result<Transaction, Box<dyn std::error::Error>> {
        let mask = self
            .data_frame
            .column(format!("{}_id", ExpensesTable::name()).as_str())?
            .i64()?
            .equal(id);

        let record = self.data_frame.filter(&mask)?.clone();
        let date = record
            .column("date")?
            .date()?
            .as_date_iter()
            .next()
            .flatten()
            .ok_or("No date!")?
            .clone();

        Ok(Transaction::Expense {
            value: record.column("value")?.f64()?.get(0).ok_or("No date!")?,
            currency: Currency::from_str(
                record
                    .column("currency")?
                    .str()?
                    .get(0)
                    .ok_or("No currency!")?,
            )?,
            date,
            category: record
                .column("category")?
                .str()?
                .get(0)
                .ok_or("No category!")?
                .to_string(),
            subcategory: record
                .column("subcategory")?
                .str()?
                .get(0)
                .ok_or("No subcategory!")?
                .to_string(),
            description: record
                .column("description")?
                .str()?
                .get(0)
                .ok_or("No description!")?
                .to_string(),
            entity_id: record
                .column("entity_id")?
                .i64()?
                .get(0)
                .ok_or("No entity_id!")?,
        })
    }
}

pub struct FundsTable {
    pub data_frame: DataFrame,
}

impl Table for FundsTable {
    fn name() -> String {
        String::from("fund_movement")
    }

    fn data_frame(&self) -> &DataFrame {
        &self.data_frame
    }

    fn mut_data_frame(&mut self) -> &mut DataFrame {
        &mut self.data_frame
    }

    fn create(data_frame: DataFrame) -> Box<Self> {
        Box::new(FundsTable { data_frame })
    }

    fn new() -> Result<Box<Self>, PolarsError> {
        let data_frame = DataFrame::new(vec![
            Column::from(Series::new(
                PlSmallStr::from(format!("{}_id", FundsTable::name())),
                Vec::<i64>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from(format!("{}_type", FundsTable::name())),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(PlSmallStr::from("value"), Vec::<f64>::new())),
            Column::from(Series::new(
                PlSmallStr::from("currency"),
                Vec::<String>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("date"),
                Vec::<NaiveDate>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("account_id"),
                Vec::<i64>::new(),
            )),
            Column::from(Series::new(PlSmallStr::from("party_id"), Vec::<i64>::new())),
        ])?;

        Ok(FundsTable::create(data_frame))
    }
}

impl FundsTable {
    /// Adds funds transaction to the table
    pub fn insert_transaction(
        &mut self,
        transaction: &Transaction,
        party_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let id: i64 = self.next_id()?;

        if let Transaction::Credit {
            value,
            currency,
            date,
            account_id,
        } = transaction
        {
            let record = df!(
                format!("{}_id", FundsTable::name()) => [id],
                format!("{}_type", FundsTable::name()) => ["Credit"], // very bad solution IMO
                "value" => [*value],
                "currency" => [currency.to_string()],
                "date" => [*date],
                "account_id" => [*account_id],
                "party_id" => [party_id]
            )?;

            self.data_frame = self.data_frame.vstack(&record)?;

            Ok(())
        } else if let Transaction::Debit {
            value,
            currency,
            date,
            account_id,
        } = transaction
        {
            let record = df!(
                format!("{}_id", FundsTable::name()) => [id],
                format!("{}_type", FundsTable::name()) => ["Debit"], // awful solution IMO
                "value" => [-1.0 * (*value)],
                "currency" => [currency.to_string()],
                "date" => [*date],
                "account_id" => [*account_id],
                "party_id" => [party_id]
            )?;

            self.data_frame = self.data_frame.vstack(&record)?;

            Ok(())
        } else {
            Err(IncorrectTableError.into())
        }
    }

    // Deletes records corresponding to a party.
    pub(crate) fn delete_party(&mut self, party_id: i64) -> Result<(), PolarsError> {
        self.data_frame = self
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").neq(lit(party_id)))
            .collect()?;

        Ok(())
    }

    /// Returns iterator of funds_ids that correspond to the given party_id
    pub(crate) fn iter_party(&self, party_id: i64) -> Result<IntoIter<i64>, PolarsError> {
        Ok(self
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").eq(lit(party_id)))
            .collect()?
            .column(format!("{}_id", FundsTable::name()).as_str())?
            .i64()?
            .into_no_null_iter()
            .collect::<Vec<i64>>()
            .into_iter())
    }

    /// Returns entity given ID
    pub(crate) fn transaction(&self, id: i64) -> Result<Transaction, Box<dyn std::error::Error>> {
        let mask = self
            .data_frame
            .column(format!("{}_id", FundsTable::name()).as_str())?
            .i64()?
            .equal(id);

        let record = self.data_frame.filter(&mask)?.clone();
        let date = record
            .column("date")?
            .date()?
            .as_date_iter()
            .next()
            .flatten()
            .ok_or("No date!")?
            .clone();
        let transaction_type = record
            .column(format!("{}_type", FundsTable::name()).as_str())?
            .str()?
            .get(0)
            .ok_or(format!("{}_type", FundsTable::name()))?
            .to_string();
        let value = record.column("value")?.f64()?.get(0).ok_or("No value!")?;
        let currency = Currency::from_str(
            record
                .column("currency")?
                .str()?
                .get(0)
                .ok_or("No currency!")?,
        )?;
        let account_id = record
            .column("account_id")?
            .i64()?
            .get(0)
            .ok_or("No account_id!")?;

        if transaction_type == String::from("Credit") {
            Ok(Transaction::Credit {
                value,
                currency,
                date,
                account_id,
            })
        } else {
            // then it is debit
            Ok(Transaction::Debit {
                value: -1.0 * value,
                currency,
                date,
                account_id,
            })
        }
    }
}

pub struct PartyTable {
    pub data_frame: DataFrame,
}

impl Table for PartyTable {
    fn name() -> String {
        String::from("party")
    }

    fn data_frame(&self) -> &DataFrame {
        &self.data_frame
    }

    fn mut_data_frame(&mut self) -> &mut DataFrame {
        &mut self.data_frame
    }

    fn create(data_frame: DataFrame) -> Box<Self> {
        Box::new(PartyTable { data_frame })
    }

    fn new() -> Result<Box<Self>, PolarsError> {
        let data_frame = DataFrame::new(vec![
            Column::from(Series::new(
                PlSmallStr::from(format!("{}_id", PartyTable::name())),
                Vec::<i64>::new(),
            )),
            Column::from(Series::new(
                PlSmallStr::from("creation_date"),
                Vec::<NaiveDate>::new(),
            )),
        ])?;

        Ok(PartyTable::create(data_frame))
    }
}

impl PartyTable {
    /// Adds party record to the table
    pub fn insert_party(&mut self, party: &Party) -> Result<(), PolarsError> {
        let id: i64 = self.next_id()?;

        let record = df!(
            format!("{}_id", PartyTable::name()) => [id],
            "creation_date" => [party.creation_date]
        )?;

        self.data_frame = self.data_frame.vstack(&record)?;

        Ok(())
    }

    // Deletes records corresponding to a party.
    pub(crate) fn delete_party(&mut self, party_id: i64) -> Result<(), PolarsError> {
        self.data_frame = self
            .data_frame
            .clone()
            .lazy()
            .filter(col("party_id").neq(lit(party_id)))
            .collect()?;

        Ok(())
    }
}

pub struct EntityTable {
    pub data_frame: DataFrame,
}

impl Table for EntityTable {
    fn name() -> String {
        String::from("entity")
    }

    fn data_frame(&self) -> &DataFrame {
        &self.data_frame
    }

    fn mut_data_frame(&mut self) -> &mut DataFrame {
        &mut self.data_frame
    }

    fn create(data_frame: DataFrame) -> Box<Self> {
        Box::new(EntityTable { data_frame })
    }

    fn new() -> Result<Box<Self>, PolarsError> {
        let data_frame: DataFrame = df!(
            format!("{}_id", EntityTable::name()) => [0i64],
            "name" => ["Unknown"],
            "country" => ["Unknown"],
            format!("{}_type", EntityTable::name()) => [EntityType::default().to_string()],
            format!("{}_subtype", EntityTable::name()) => [""],
            "creation_date" => [Local::now().date_naive()])?;

        Ok(EntityTable::create(data_frame))
    }
}

impl EntityTable {
    /// Iterator over IDs
    pub(crate) fn iter(&self) -> Result<IntoIter<i64>, PolarsError> {
        Ok(self
            .data_frame
            .sort(["name"], Default::default())?
            .column(format!("{}_id", EntityTable::name()).as_str())?
            .i64()?
            .into_no_null_iter()
            .collect::<Vec<i64>>()
            .into_iter())
    }

    /// Adds entity to the table
    pub fn insert_entity(&mut self, entity: &Entity) -> Result<i64, PolarsError> {
        let id: i64 = self.next_id()?;

        let record = df!(
            format!("{}_id", EntityTable::name()) => [id],
            "name" => [entity.name()],
            "country" => [entity.country()],
            format!("{}_type", EntityTable::name()) => [entity.entity_type().to_string()],
            format!("{}_subtype", EntityTable::name()) => [entity.entity_subtype()],
            "creation_date" => [Local::now().date_naive()]
        )?;

        self.data_frame = self.data_frame.vstack(&record)?;

        Ok(id)
    }

    /// Returns entity given ID
    pub(crate) fn entity(&self, id: i64) -> Result<Entity, Box<dyn std::error::Error>> {
        let mask = self
            .data_frame
            .column(format!("{}_id", EntityTable::name()).as_str())?
            .i64()?
            .equal(id);

        let record = self.data_frame.filter(&mask)?;

        Ok(Entity::new(
            record
                .column("name")?
                .str()?
                .get(0)
                .ok_or("No name!")?
                .to_string(),
            record
                .column("country")?
                .str()?
                .get(0)
                .ok_or("No country!")?
                .to_string(),
            EntityType::from_str(
                record
                    .column(format!("{}_type", EntityTable::name()).as_str())?
                    .str()?
                    .get(0)
                    .ok_or(format!("No {}_type!", EntityTable::name()))?,
            )?,
            record
                .column(format!("{}_subtype", EntityTable::name()).as_str())?
                .str()?
                .get(0)
                .ok_or(format!("No {}_subtype", EntityTable::name()))?
                .to_string(),
        ))
    }

    /// Returns list of unique countries
    pub(crate) fn countries(&self) -> Result<Vec<String>, PolarsError> {
        Ok(self
            .data_frame()
            .column("country")?
            .unique()?
            .str()?
            .sort(false)
            .into_no_null_iter()
            .map(|s| s.to_string())
            .collect())
    }

    pub(crate) fn subtypes(&self) -> Result<Vec<String>, PolarsError> {
        // filter type?
        Ok(self
            .data_frame()
            .column(format!("{}_subtype", EntityTable::name()).as_str())?
            .unique()?
            .str()?
            .sort(false)
            .into_no_null_iter()
            .map(|s| s.to_string())
            .collect())
    }
}
pub struct AccountTable {
    pub data_frame: DataFrame,
}

impl Table for AccountTable {
    fn name() -> String {
        String::from("account")
    }

    fn data_frame(&self) -> &DataFrame {
        &self.data_frame
    }

    fn mut_data_frame(&mut self) -> &mut DataFrame {
        &mut self.data_frame
    }

    fn create(data_frame: DataFrame) -> Box<Self> {
        Box::new(AccountTable { data_frame })
    }

    fn new() -> Result<Box<Self>, PolarsError> {
        let data_frame: DataFrame = df!(
            format!("{}_id", AccountTable::name()) => [0i64],
            "name" => ["Unknown"],
            "country" => ["Unknown"],
            "currency" => [Currency::default().to_string()],
            format!("{}_type", AccountTable::name()) => [AccountType::default().to_string()],
            "initial_balance" => [0.0f64],
            "creation_date" => [Local::now().date_naive()])?;

        Ok(AccountTable::create(data_frame))
    }
}

impl AccountTable {
    /// Iterator over IDs
    pub(crate) fn iter(&self) -> Result<IntoIter<i64>, PolarsError> {
        Ok(self
            .data_frame
            .sort(["name"], Default::default())?
            .column(format!("{}_id", AccountTable::name()).as_str())?
            .i64()?
            .into_no_null_iter()
            .collect::<Vec<i64>>()
            .into_iter())
    }

    /// Adds account record to the table
    pub fn insert_account(&mut self, account: &Account) -> Result<i64, PolarsError> {
        let id: i64 = self.next_id()?;

        let record = df!(
            format!("{}_id", AccountTable::name()) => [id],
            "name" => [account.name()],
            "country" => [account.country()],
            "currency" => [account.currency().to_string()],
            format!("{}_type", AccountTable::name()) => [account.account_type().to_string()],
            "initial_balance" => [account.initial_balance()],
            "creation_date" => [Local::now().date_naive()]
        )?;

        self.data_frame = self.data_frame.vstack(&record)?;

        Ok(id)
    }

    /// Retrieves account from the table, given ID
    pub(crate) fn account(&self, id: i64) -> Result<Account, Box<dyn std::error::Error>> {
        let mask = self
            .data_frame
            .column(format!("{}_id", AccountTable::name()).as_str())?
            .i64()?
            .equal(id);

        let record = self.data_frame.filter(&mask)?;

        Ok(Account::new(
            record
                .column("name")?
                .str()?
                .get(0)
                .ok_or("No name!")?
                .to_string(),
            record
                .column("country")?
                .str()?
                .get(0)
                .ok_or("No country!")?
                .to_string(),
            Currency::from_str(
                record
                    .column("currency")?
                    .str()?
                    .get(0)
                    .ok_or("No currency!")?,
            )?,
            AccountType::from_str(
                record
                    .column(format!("{}_type", AccountTable::name()).as_str())?
                    .str()?
                    .get(0)
                    .ok_or(format!("No {}_type!", AccountTable::name()))?,
            )?,
            record
                .column("initial_balance")?
                .f64()?
                .get(0)
                .ok_or("No initial_balance!")?,
        ))
    }

    pub(crate) fn countries(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(self
            .data_frame()
            .column("country")?
            .unique()?
            .str()?
            .sort(false)
            .into_no_null_iter()
            .map(|s| s.to_string())
            .collect())
    }
}
