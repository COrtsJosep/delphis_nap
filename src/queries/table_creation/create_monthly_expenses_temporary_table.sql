create table if not exists monthly_expenses_temporary (
	month text not null,
	category text not null,
	value real not null
)
