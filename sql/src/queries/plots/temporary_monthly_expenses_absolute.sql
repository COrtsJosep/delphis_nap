create temporary table monthly_expenses_temporary
as 
	select 
		strftime('%y-%m', currency_exchanges_from_eur.date) as month,
		expenses.category as category,
		sum(
			(case when expenses.value is null then 0.0 else expenses.value end)
			* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
			* (case when currency_exchanges_from_eur.value is null then 1.0 else currency_exchanges_from_eur.value end) 
		) as value 
	from
		currency_exchanges as currency_exchanges_to_eur
		left join currency_exchanges as currency_exchanges_from_eur
		on
			currency_exchanges_from_eur.date = currency_exchanges_to_eur.date
			and currency_exchanges_from_eur.currency_to = ?
			and currency_exchanges_from_eur.currency_from = currency_exchanges_to_eur.currency_to	
		left join expenses
		on 
			expenses.date = currency_exchanges_from_eur.date
			and expenses.currency = currency_exchanges_from_eur.currency_to
	where
		currency_exchanges_to_eur.date >= (select min(date) from expenses)
		and currency_exchanges_to_eur.date <= (select max(date) from expenses) 
	group by 
		strftime('%y-%m', currency_exchanges_from_eur.date),
		expenses.category
