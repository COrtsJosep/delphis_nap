select
	'Total' as category,
	'Total' as subcategory,
	sum(value) as "value!",
	sum(value) / ? as "value_day!",
	sum(value) / sum(value) as "value_total_expenses!",
	sum(value) / ? as "value_total_incomes!"
from expenses_temporary

