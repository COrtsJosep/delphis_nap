insert into currency_exchanges 
(currency_exchange_id, date_from, date_to, currency_from, currency_to, value)
with ecb_exchanges_windowed as (
	select 
		currency_exchange_id,
		date as date_from,
		lead(date, 1, NULL) over (order by date) as date_to,
		coalesce(
			value,  -- value for almost all rows 
			lead(value, 1, NULL) over (order by date)  -- value for first row
		) as value
   	from (
   		select *
   		from ecb_exchanges
		where 
			currency_from = ?
			and currency_to = ?
		union select concat('1900-01-01', '_', ?, '_', ?), '1900-01-01', ?, ?, null
		union select null, '3000-12-31', ?, ?, null
	)
)
select
	currency_exchange_id,
	date_from,
	date_to,
	?,  -- currency_from
	?,  -- currency_to
	value
from ecb_exchanges_windowed
where currency_exchange_id is not null -- filter out last row (3000-12-31)

	        


