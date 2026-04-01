create table if not exists entities (
	entity_id integer primary key,
	name text not null,
	country text not null,
	entity_type text not null,
	entity_subtype text not null,
	creation_date text not null
)
