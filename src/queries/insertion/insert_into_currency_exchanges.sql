insert into currency_exchanges 
(currency_exchange_id, date, currency_from, currency_to, value)
with ecb_exchanges_nogaps as (
	select 
		dates.date,
		ecb_exchanges.value,
		count(ecb_exchanges.value) over (order by dates.date) as value_count
   	from dates
	left join ecb_exchanges
		on dates.date = ecb_exchanges.date
		and ecb_exchanges.currency_from = ?
		and ecb_exchanges.currency_to = ?
	where dates.date <= ?
)
select
	concat(
		date,
		'_',
		?, -- currency_from
		'_',
		?  -- currency_to
	),
	date,
	?,  -- currency_from
	?,  -- currency_to
	first_value(value) over (partition by value_count order by date) as value
from ecb_exchanges_nogaps

	        


