create table if not exists currency_exchanges (
	currency_exchange_id text primary key,
	date_from text not null,
	date_to text not null,
	currency_from text not null,
	currency_to text not null,
	value real not null
)
