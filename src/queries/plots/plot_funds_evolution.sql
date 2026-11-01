select 
	dates.date as "date!",
	sum(  -- outer sum: does the cumsum
		sum(  -- inner sum: does the groupby sum by date
			(coalesce(fund_movements.value, 0.0) + coalesce(accounts.initial_balance, 0.0))
			* coalesce(currency_exchanges_to_eur.value, 1.0)
			* coalesce(currency_exchanges_from_eur.value, 1.0)
		)
	) over (order by dates.date) as "value!"	
from	
	dates
	cross join (select distinct currency_from as currency from currency_exchanges) as currencies
	left join currency_exchanges as currency_exchanges_to_eur
	on
		currency_exchanges_to_eur.date_from <= dates.date 
		and dates.date < currency_exchanges_to_eur.date_to
		and currency_exchanges_to_eur.currency_to = 'EUR'
		and currency_exchanges_to_eur.currency_from = currencies.currency
	left join currency_exchanges as currency_exchanges_from_eur
	on
		currency_exchanges_from_eur.date_from <= dates.date
		and dates.date < currency_exchanges_from_eur.date_to
		and currency_exchanges_from_eur.currency_to = ?
		and currency_exchanges_from_eur.currency_from = 'EUR'	
	left join (select creation_date, currency, sum(initial_balance) as initial_balance from accounts group by creation_date, currency) as accounts
	on 
		accounts.creation_date = dates.date
		and accounts.currency = currencies.currency
	left join (select date, currency, sum(value) as value from fund_movements group by date, currency) as fund_movements
	on
		fund_movements.date = dates.date
		and fund_movements.currency = currencies.currency
where
	dates.date >= min((select min(creation_date) from accounts), (select min(date) from fund_movements))
	and dates.date <= max((select max(creation_date) from accounts), (select max(date) from fund_movements))
group by dates.date
order by dates.date asc
