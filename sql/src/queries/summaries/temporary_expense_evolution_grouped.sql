create temporary table expense_evolution_temporary
as 
	select
		strftime(?, expenses.date) as date,
		expenses.category,
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
	group by
		strftime(?, expenses.date),
		expenses.category
	order by
		strftime(?, expenses.date) asc,
		expenses.category asc
