select 
	sum(
		incomes.value
		* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
		* (case when currency_exchanges_from_eur.value is null then 1.0 else currency_exchanges_from_eur.value end) 
	) as "total_income?: f64"
from 
	incomes
	left join currency_exchanges as currency_exchanges_to_eur
	on 
		currency_exchanges_to_eur.date = incomes.date 
		and currency_exchanges_to_eur.currency_to = 'EUR'
		and currency_exchanges_to_eur.currency_from = incomes.currency
	left join currency_exchanges as currency_exchanges_from_eur
	on
		currency_exchanges_from_eur.date = min((select max(date) from currency_exchanges), ?)
		and currency_exchanges_from_eur.currency_to = ?
		and currency_exchanges_from_eur.currency_from = currency_exchanges_to_eur.currency_to
where 
	? <= incomes.date 
	and incomes.date <= ?
