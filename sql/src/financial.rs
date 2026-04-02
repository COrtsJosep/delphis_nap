use chrono::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::slice::Iter;
use strum_macros::{EnumIter, EnumString};

/// A party is a balanced set of accounting transactions that happened together and that are
/// related to each other.
/// I know "party" is not the right word for that, but it's the literal translation from
/// Spanish ("partida financiera"), I cannot think of a better name, and adds a festive
/// touch to the code.
pub struct Party {
    pub transactions: Vec<Transaction>,
    pub creation_date: NaiveDate,
}

impl Party {
    pub fn new(transactions: Vec<Transaction>) -> Party {
        Party {
            transactions,
            creation_date: Local::now().date_naive(),
        }
    }

    /// Checks whether the party is balanced. This means that for every currency involved in
    /// the party, the amount expended/earned is associated with a decrease/increase of funds
    /// of the same amount and opposite sign.
    /// For instance, a bill of 50€ grocery shopping has to be balanced with, let's say, a 50€
    /// withdrawal from a bank account. The relationship does not need to be 1:1, for
    /// instance, a 350 SEK bill for clothing and a 230 SEK bill for presents can be balanced
    /// with a 500 SEK withdrawal from a bank account and an 80 SEK withdrawal from pocket money.
    pub(crate) fn is_valid(&self) -> bool {
        let mut aggregates: HashMap<&Currency, f64> = HashMap::new();

        for transaction in &self.transactions {
            let value: f64 = transaction.value() * transaction.sign();
            let currency: &Currency = transaction.currency();

            aggregates
                .entry(currency)
                .and_modify(|aggregate: &mut f64| *aggregate += value)
                .or_insert(value);
        }

        for (_, val) in aggregates.iter() {
            if (*val).abs() >= 0.01 {
                return false;
            } // one or more cents off
        }

        // return true if party is balanced and is nonempty
        !self.transactions.is_empty()
    }

    /// Adds a new transaction to the party.
    pub(crate) fn add_transaction(&mut self, transaction: Transaction) -> () {
        self.transactions.push(transaction);
    }

    pub(crate) fn iter(&mut self) -> Iter<'_, Transaction> {
        self.transactions.iter()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    pub(crate) fn remove(&mut self, i: usize) -> Transaction {
        self.transactions.remove(i)
    }
}

impl Default for Party {
    fn default() -> Party {
        Party::new(Vec::<Transaction>::new())
    }
}

#[derive(Debug, Hash, PartialEq, Eq, EnumIter)]
pub enum TransactionType {
    Income,
    Expense,
    Credit,
    Debit,
}

impl TransactionType {
    pub(crate) fn clone(&self) -> TransactionType {
        match self {
            TransactionType::Income => TransactionType::Income,
            TransactionType::Expense => TransactionType::Expense,
            TransactionType::Credit => TransactionType::Credit,
            TransactionType::Debit => TransactionType::Debit,
        }
    }

    pub(crate) fn is_fund_change(&self) -> bool {
        match self {
            TransactionType::Income | TransactionType::Expense => false,
            TransactionType::Credit | TransactionType::Debit => true,
        }
    }
}

// Conversion to string
impl Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TransactionType::Income => "Income".to_string(),
            TransactionType::Expense => "Expense".to_string(),
            TransactionType::Credit => "Credit".to_string(),
            TransactionType::Debit => "Debit".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Default for TransactionType {
    fn default() -> Self {
        TransactionType::Income
    }
}

/// Basic entity of the accounting system. Incomes and expenses reflect what event provoked
/// the movement, credit and debit record what funds were used.
#[derive(Clone)]
pub enum Transaction {
    Income {
        value: f64,
        currency: Currency,
        date: NaiveDate,
        category: String,    // salary, interest
        subcategory: String, // regular salary, 13-month salary
        description: String,
        entity_id: i64,
    },
    Expense {
        value: f64,
        currency: Currency,
        date: NaiveDate,
        category: String,    // utilities, rent, transport
        subcategory: String, // train, bus, hairdresser
        description: String,
        entity_id: i64,
    },
    Credit {
        value: f64,
        currency: Currency,
        date: NaiveDate,
        account_id: i64,
    },
    Debit {
        value: f64,
        currency: Currency,
        date: NaiveDate,
        account_id: i64,
    },
}

impl Transaction {
    /// Sign getter.
    fn sign(&self) -> f64 {
        match self {
            Transaction::Income { .. } => 1.0,
            Transaction::Expense { .. } => -1.0,
            Transaction::Credit { .. } => -1.0,
            Transaction::Debit { .. } => 1.0,
        }
    }

    /// Value getter.
    pub(crate) fn value(&self) -> f64 {
        match self {
            Transaction::Income { value, .. }
            | Transaction::Expense { value, .. }
            | Transaction::Credit { value, .. }
            | Transaction::Debit { value, .. } => *value,
        }
    }

    /// Currency getter.
    pub(crate) fn currency(&self) -> &Currency {
        match self {
            Transaction::Income { currency, .. }
            | Transaction::Expense { currency, .. }
            | Transaction::Credit { currency, .. }
            | Transaction::Debit { currency, .. } => currency,
        }
    }

    /// Date getter.
    pub(crate) fn date(&self) -> &NaiveDate {
        match self {
            Transaction::Income { date, .. }
            | Transaction::Expense { date, .. }
            | Transaction::Credit { date, .. }
            | Transaction::Debit { date, .. } => date,
        }
    }

    /// Type getter.
    pub(crate) fn transaction_type(&self) -> String {
        match self {
            Transaction::Income { .. } => String::from("Income"),
            Transaction::Expense { .. } => String::from("Expense"),
            Transaction::Credit { .. } => String::from("Credit"),
            Transaction::Debit { .. } => String::from("Debit"),
        }
    }
}

/// Conversion to string
impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Transaction::Income {
                value,
                currency,
                date,
                category,
                subcategory,
                ..
            } => format!("Income ({category}, {subcategory}): {currency} {value}, at date {date}"),
            Transaction::Expense {
                value,
                currency,
                date,
                category,
                subcategory,
                ..
            } => format!("Expense ({category}, {subcategory}): {currency} {value}, at date {date}"),
            Transaction::Credit {
                value,
                currency,
                date,
                ..
            } => format!("Credit: {currency} {value}, at date {date}"),
            Transaction::Debit {
                value,
                currency,
                date,
                ..
            } => format!("Debit: {currency} {value}, at date {date}"),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, EnumIter, Clone, EnumString)]
pub enum Currency {
    EUR,
    CHF,
    SEK,
}

impl Currency {
    pub(crate) fn clone(&self) -> Currency {
        match self {
            Currency::EUR { .. } => Currency::EUR,
            Currency::CHF { .. } => Currency::CHF,
            Currency::SEK { .. } => Currency::SEK,
        }
    }
}

// Conversion to string
impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Currency::EUR => "EUR".to_string(),
            Currency::CHF => "CHF".to_string(),
            Currency::SEK => "SEK".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Default for Currency {
    fn default() -> Self {
        Currency::EUR
    }
}

/// Entity to which the expense is paid or, alternatively, that hands in the income.
pub struct Entity {
    name: String,
    country: String,
    entity_type: EntityType,
    entity_subtype: String, //supermarket, pharmacy, ... (?)
}

impl Entity {
    pub(crate) fn name(&self) -> String {
        self.name.to_string()
    }
    pub(crate) fn country(&self) -> String {
        self.country.to_string()
    }
    pub(crate) fn entity_type(&self) -> &EntityType {
        &self.entity_type
    }
    pub(crate) fn entity_subtype(&self) -> String {
        self.entity_subtype.to_string()
    }

    pub fn new(
        name: String,
        country: String,
        entity_type: EntityType,
        entity_subtype: String,
    ) -> Self {
        Self {
            name,
            country,
            entity_type,
            entity_subtype,
        }
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!("{} ({})", self.name.clone(), self.country.clone());
        write!(f, "{}", str)
    }
}

/// Account where funds are stored.
pub struct Account {
    name: String,
    country: String,
    currency: Currency,
    account_type: AccountType,
    initial_balance: f64,
}

impl Account {
    pub(crate) fn name(&self) -> String {
        self.name.to_string()
    }
    pub(crate) fn country(&self) -> String {
        self.country.to_string()
    }
    pub(crate) fn currency(&self) -> &Currency {
        &self.currency
    }
    pub(crate) fn account_type(&self) -> &AccountType {
        &self.account_type
    }
    pub(crate) fn initial_balance(&self) -> f64 {
        self.initial_balance
    }

    pub fn new(
        name: String,
        country: String,
        currency: Currency,
        account_type: AccountType,
        initial_balance: f64,
    ) -> Self {
        Self {
            name,
            country,
            currency,
            account_type,
            initial_balance,
        }
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!(
            "{} ({}, {})",
            self.name,
            self.country,
            self.currency.to_string()
        );
        write!(f, "{}", str)
    }
}

#[derive(Debug, EnumIter, PartialEq, EnumString)]
pub enum EntityType {
    Firm,
    Human,
    State,
    NGO,
}

impl EntityType {
    pub(crate) fn clone(&self) -> EntityType {
        match self {
            EntityType::Firm { .. } => EntityType::Firm,
            EntityType::Human { .. } => EntityType::Human,
            EntityType::State { .. } => EntityType::State,
            EntityType::NGO { .. } => EntityType::NGO,
        }
    }
}

/// Conversion to string
impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            EntityType::Firm { .. } => "Firm".to_string(),
            EntityType::Human { .. } => "Human".to_string(),
            EntityType::State { .. } => "State".to_string(),
            EntityType::NGO { .. } => "NGO".to_string(),
        };
        write!(f, "{}", str)
    }
}
impl Default for EntityType {
    fn default() -> Self {
        EntityType::Firm
    }
}

#[derive(Debug, EnumIter, PartialEq, EnumString)]
pub enum AccountType {
    Deposit,
    Investment,
    Cash,
}

impl AccountType {
    pub(crate) fn clone(&self) -> AccountType {
        match self {
            AccountType::Deposit { .. } => AccountType::Deposit,
            AccountType::Investment { .. } => AccountType::Investment,
            AccountType::Cash { .. } => AccountType::Cash,
        }
    }
}

/// Conversion to string
impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AccountType::Deposit { .. } => "Deposit".to_string(),
            AccountType::Investment { .. } => "Investment".to_string(),
            AccountType::Cash { .. } => "Cash".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Deposit
    }
}
