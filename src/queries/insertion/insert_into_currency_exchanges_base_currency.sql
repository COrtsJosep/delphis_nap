insert into currency_exchanges 
(currency_exchange_id, date, currency_from, currency_to, value)
select
	concat(dates.date, '_', ?, '_', ?) as currency_exchange_id,
	dates.date,
	?,  -- currency_from
	?,  -- currency_to
	1.0
from dates
