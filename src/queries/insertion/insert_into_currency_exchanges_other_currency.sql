insert into currency_exchanges 
(currency_exchange_id, date, currency_from, currency_to, value)
with ecb_exchanges_windowed as (
	select 
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
		union select null, '1900-01-01', ?, ?, null
		union select null, '3000-12-31', ?, ?, null
	)
)
select
	concat(dates.date, '_', ?, '_', ?) as currency_exchange_id,
	dates.date,
	?,  -- currency_from
	?,  -- currency_to
	ecb_exchanges_windowed.value
from dates
left join ecb_exchanges_windowed
	on ecb_exchanges_windowed.date_from <= dates.date
	and dates.date < ecb_exchanges_windowed.date_to
	and ecb_exchanges_windowed.date_to is not null -- filter out last row (3000-12-31)
