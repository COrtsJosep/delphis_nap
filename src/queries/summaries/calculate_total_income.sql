select 
	sum(
		incomes.value
		* currency_exchanges_to_eur.value
		* currency_exchanges_from_eur.value
	) as "total_income?: f64"
from incomes
	inner join currency_exchanges as currency_exchanges_to_eur
		on currency_exchanges_to_eur.date = incomes.date
		and currency_exchanges_to_eur.currency_to = 'EUR'
		and currency_exchanges_to_eur.currency_from = incomes.currency
	inner join currency_exchanges as currency_exchanges_from_eur
		on currency_exchanges_from_eur.date = incomes.date
		and currency_exchanges_from_eur.currency_to = ?
		and currency_exchanges_from_eur.currency_from = currency_exchanges_to_eur.currency_to
where 
	? <= incomes.date 
	and incomes.date <= ?
