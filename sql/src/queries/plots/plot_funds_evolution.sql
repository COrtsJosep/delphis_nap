select 
	base.date as "date!",
	sum(  -- outer sum: does the cumsum
		sum(  -- inner sum: does the groupby sum by date
			(
				case when fund_movements.value is null then 0.0 else fund_movements.value end 
					+ case when accounts.initial_balance is null then 0.0 else accounts.initial_balance end
			)
			* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
			* (case when currency_exchanges_from_eur.value is null then 1.0 else currency_exchanges_from_eur.value end) 
		)
	) over (order by base.date) as "value!"	
from	
	(select distinct date, currency_to as currency from currency_exchanges) as base
	left join (select creation_date, currency, sum(initial_balance) as initial_balance from accounts group by creation_date, currency) as accounts
	on 
		accounts.creation_date = base.date
		and accounts.currency = base.currency
	left join (select date, currency, sum(value) as value from fund_movements group by date, currency) as fund_movements
	on
		fund_movements.date = base.date
		and fund_movements.currency = base.currency
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
where
	base.date >= min((select min(creation_date) from accounts), (select min(date) from fund_movements))
	and base.date <= max((select max(creation_date) from accounts), (select max(date) from fund_movements))
group by base.date
order by base.date asc
