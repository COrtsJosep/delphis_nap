create temporary table expenses_temporary
as 
	select
		expenses.category,
		expenses.subcategory,
		sum(
			expenses.value
			* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
			* (case when currency_exchanges_from_eur.value is null then 1.0 else currency_exchanges_from_eur.value end) 
		) as value
	from
		expenses
		left join currency_exchanges as currency_exchanges_to_eur
		on 
			currency_exchanges_to_eur.date = expenses.date 
			and currency_exchanges_to_eur.currency_to = 'EUR'
			and currency_exchanges_to_eur.currency_from = expenses.currency
		left join currency_exchanges as currency_exchanges_from_eur
		on
			currency_exchanges_from_eur.date = date('now')
			and currency_exchanges_from_eur.currency_to = ?
			and currency_exchanges_from_eur.currency_from = currency_exchanges_to_eur.currency_to	
	where ? <= expenses.date and expenses.date <= ?
	group by 
		expenses.category,
		expenses.subcategory
	order by
		expenses.category asc,
		expenses.subcategory asc
