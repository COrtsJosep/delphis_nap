create temporary table monthly_expenses_temporary
as 
	select 
		strftime('%Y-%m', dates.date) as month,
		expenses.category as category,
		sum(
			(case when expenses.value is null then 0.0 else expenses.value end)
			* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
			* (case when currency_exchanges_from_eur.value is null then 1.0 else currency_exchanges_from_eur.value end) 
		) as value 
	from
		dates
		left join currency_exchanges as currency_exchanges_to_eur
		on
			currency_exchanges_to_eur.date_from <= dates.date < currency_exchanges_to_eur.date_to
			and currency_exchanges_to_eur.currency_to = 'EUR'
		left join currency_exchanges as currency_exchanges_from_eur
		on
			currency_exchanges_from_eur.date_from <= dates.date < currency_exchanges_from_eur.date_to
			and currency_exchanges_from_eur.currency_to = ?
			and currency_exchanges_from_eur.currency_from = 'EUR'
		left join expenses
		on 
			expenses.date = dates.date
			and expenses.currency = currency_exchanges_to_eur.currency_from
	where
		(select min(date) from expenses) <= dates.date <= (select max(date) from expenses)
		and expenses.category is not null
	group by 
		strftime('%Y-%m', dates.date),
		expenses.category
