create table if not exists currency_exchanges (
	currency_expense_id text primary key,
	date text not null,
	currency_from text not null,
	currency_to text not null,
	value real not null
)
