select
	transaction_type,
	date,
	value,
	currency,
	name,
	category,
	subcategory,
	party_id
from (
		select
			date,
			value,
			currency,
			category,
			subcategory,
			entity_id,
			party_id,
			"Income" as transaction_type 
		from incomes
		union all	
		select
			date,
			value,
			currency,
			category,
			subcategory,
			entity_id,
			party_id,
			"Expense" as transaction_type 
		from expenses
	) as transactions 
	inner join entities
	on transactions.entity_id = entities.entity_id
order by 
	date asc,
	party_id asc
limit ?
