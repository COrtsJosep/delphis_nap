create temporary table monthly_expenses_temporary
as 
	select 
		strftime('%Y-%m', base.date) as month,
		expenses.category as category,
		sum(
			(case when expenses.value is null then 0.0 else expenses.value end)
			* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
			* (case when currency_exchanges_from_eur.value is null then 1.0 else currency_exchanges_from_eur.value end) 
		) as value 
	from
		(select distinct date, currency_to as currency from currency_exchanges) as base
		left join currency_exchanges as currency_exchanges_to_eur
		on
			currency_exchanges_to_eur.date = base.date
			and currency_exchanges_to_eur.currency_from = base.currency
			and currency_exchanges_to_eur.currency_to = 'EUR'
		left join currency_exchanges as currency_exchanges_from_eur
		on
			currency_exchanges_from_eur.date = base.date
			and currency_exchanges_from_eur.currency_to = ?
			and currency_exchanges_from_eur.currency_from = 'EUR'
		left join expenses
		on 
			expenses.date = base.date
			and expenses.currency = base.currency
	where
		base.date >= (select min(date) from expenses)
		and base.date <= (select max(date) from expenses)
		and expenses.category is not null
	group by 
		strftime('%Y-%m', base.date),
		expenses.category
