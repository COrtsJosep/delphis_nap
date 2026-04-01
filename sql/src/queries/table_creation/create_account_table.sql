create table if not exists accounts (
	account_id integer primary key,
	name text not null,
	country text not null,
	currency text not null,
	account_type text not null,
	initial_balance real not null,
	creation_date text not null
)
