select 
	sum(incomes.value * currency_exchanges.value) as total_income 
from 
	incomes
	left join currency_exchanges 
	on 
		incomes.date = currency_exchanges.date 
		and incomes.currency = currency_exchanges.currency_from
where 
	? <= incomes.date 
	and incomes.date <= ? 
	and currency_exchanges.currency_to = ?
