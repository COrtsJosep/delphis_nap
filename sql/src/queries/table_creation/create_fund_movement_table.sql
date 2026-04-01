create table if not exists fund_movements (
	fund_movement_id integer primary key,
	fund_movement_type text not null,
	value real not null,
	currency text not null,
	date text not null,
	account_id not null,
	party_id not null,
	foreign key(account_id) references accounts(account_id),
	foreign key(party_id) references parties(party_id)
)
