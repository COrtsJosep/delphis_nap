select
	category as "category!",
	subcategory as "subcategory!",
	value as "value!",
	value / ? as "value_day!",
	value / sum(value) as "value_total_expenses!",
	value / ? as "value_total_incomes!"
from expenses_temporary
	
