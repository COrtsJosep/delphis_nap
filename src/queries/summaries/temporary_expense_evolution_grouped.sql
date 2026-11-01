create temporary table expense_evolution_temporary
as 
	select
		strftime(?, dates.date) as date,
		expenses.category,
		sum(
			expenses.value
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
		strftime(?, dates.date),
		expenses.category
	order by
		strftime(?, dates.date) asc,
		expenses.category asc
