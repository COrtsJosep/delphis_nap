create table if not exists expenses_temporary (
	category text not null,
	subcategory text not null,
	value real not null,
	value_day real not null,
	value_total_expenses real not null,
	value_total_incomes real not null
)
