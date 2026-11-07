select 
	currency_exchanges_to_eur.date as "date!",
	sum(  -- outer sum: does the cumsum
		sum(  -- inner sum: does the groupby sum by date
			coalesce(fund_movements.value, 0.0)
			* currency_exchanges_to_eur.value
			* currency_exchanges_from_eur.value
		)
	) over (order by currency_exchanges_to_eur.date) as "value!: f64"	
from
	currency_exchanges as currency_exchanges_to_eur
	inner join currency_exchanges as currency_exchanges_from_eur
		on
			currency_exchanges_from_eur.date = currency_exchanges_to_eur.date
			and currency_exchanges_from_eur.currency_to = ?
			and currency_exchanges_from_eur.currency_from = 'EUR'
	left join (
			select creation_date as date, currency, initial_balance as value from accounts
			union all
			select date, currency, value from fund_movements
		) as fund_movements
		on
			fund_movements.date = currency_exchanges_to_eur.date
			and fund_movements.currency = currency_exchanges_to_eur.currency_from
where
	currency_exchanges_to_eur.currency_to = 'EUR'
	and currency_exchanges_to_eur.date >= min((select min(creation_date) from accounts), (select min(date) from fund_movements))
	and currency_exchanges_to_eur.date <= max((select max(creation_date) from accounts), (select max(date) from fund_movements))
group by currency_exchanges_to_eur.date
order by currency_exchanges_to_eur.date asc
