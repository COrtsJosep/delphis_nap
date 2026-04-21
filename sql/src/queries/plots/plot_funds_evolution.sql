select 
	currency_exchanges_to_eur.date as "date!",
	sum(  -- outer sum: does the cumsum
		sum(  -- inner sum: does the groupby sum by date
			(
				case when fund_movements.value is null then 0.0 else fund_movements.value end 
					+ case when accounts.initial_balance is null then 0.0 else accounts.initial_balance end
			)
			* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
			* (case when currency_exchanges_from_eur.value is null then 1.0 else currency_exchanges_from_eur.value end) 
		)
	) over (order by currency_exchanges_to_eur.date) as "value!"	
from
	currency_exchanges as currency_exchanges_to_eur
	left join currency_exchanges as currency_exchanges_from_eur
	on
		currency_exchanges_from_eur.date = currency_exchanges_to_eur.date
		and currency_exchanges_from_eur.currency_to = ?
		and currency_exchanges_from_eur.currency_from = currency_exchanges_to_eur.currency_to	
	left join accounts
	on 
		accounts.creation_date = currency_exchanges_from_eur.date
		and accounts.currency = currency_exchanges_from_eur.currency_to
	left join fund_movements 
	on
		fund_movements.date = currency_exchanges_from_eur.date
		and fund_movements.currency = currency_exchanges_from_eur.currency_to
where
	currency_exchanges_to_eur.date >= min((select min(creation_date) from accounts), (select min(date) from fund_movements))
	and currency_exchanges_to_eur.date <= max((select max(creation_date) from accounts), (select max(date) from fund_movements)) 
group by currency_exchanges_to_eur.date
order by currency_exchanges_to_eur.date asc
