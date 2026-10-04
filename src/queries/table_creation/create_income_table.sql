create table if not exists incomes (
	income_id integer primary key,
	value real not null,
	currency text not null,
	date text not null,
	category text not null,
	subcategory text not null,
	description text not null,
	entity_id integer not null,
	party_id integer not null,
	foreign key(entity_id) references entities(entity_id),
	foreign key(party_id) references parties(party_id)
)
