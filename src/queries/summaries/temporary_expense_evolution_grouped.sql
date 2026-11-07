create temporary table expense_evolution_temporary
as 
	select
		strftime(?, currency_exchanges_to_eur.date) as date,
		expenses.category,
		sum(
			expenses.value
			* currency_exchanges_to_eur.value
			* currency_exchanges_from_eur.value
		) as value
	from currency_exchanges as currency_exchanges_to_eur
		inner join currency_exchanges as currency_exchanges_from_eur
			on currency_exchanges_from_eur.date = currency_exchanges_to_eur.date
			and currency_exchanges_from_eur.currency_to = ?
			and currency_exchanges_from_eur.currency_from = 'EUR'
		left join expenses
			on expenses.date = currency_exchanges_to_eur.date
			and expenses.currency = currency_exchanges_to_eur.currency_from
	where
		currency_exchanges_to_eur.currency_to = 'EUR'
		and (select min(date) from expenses) <= currency_exchanges_to_eur.date
		and currency_exchanges_to_eur.date <= (select max(date) from expenses)
		and expenses.category is not null
	group by 
		strftime(?, currency_exchanges_to_eur.date),
		expenses.category
	order by
		strftime(?, currency_exchanges_to_eur.date) asc,
		expenses.category asc
