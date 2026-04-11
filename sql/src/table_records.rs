use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct AccountRecord {
    pub account_id: i64,
    pub name: String,
    pub country: String,
    pub currency: String,
    pub account_type: String,
    pub initial_balance: f64,
    pub creation_date: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EntityRecord {
    pub entity_id: i64,
    pub name: String,
    pub country: String,
    pub entity_type: String,
    pub entity_subtype: String,
    pub creation_date: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ExpenseRecord {
    pub expense_id: i64,
    pub value: f64,
    pub currency: String,
    pub date: String,
    pub category: String,
    pub subcategory: String,
    pub description: String,
    pub entity_id: i64,
    pub party_id: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FundMovementRecord {
    pub fund_movement_id: i64,
    pub fund_movement_type: String,
    pub value: f64,
    pub currency: String,
    pub date: String,
    pub account_id: i64,
    pub party_id: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IncomeRecord {
    pub income_id: i64,
    pub value: f64,
    pub currency: String,
    pub date: String,
    pub category: String,
    pub subcategory: String,
    pub description: String,
    pub entity_id: i64,
    pub party_id: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PartyRecord {
    pub party_id: i64,
    pub creation_date: String,
}
