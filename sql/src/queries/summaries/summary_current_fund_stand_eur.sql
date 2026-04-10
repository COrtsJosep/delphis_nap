select
	accounts.name,
	accounts.country,
	'EUR' as currency,
	accounts.account_type,
	(accounts.initial_balance + fund_changes.value) * (case when currency_exchanges.value is null then 1.0 else currency_exchanges.value end) as "current_value!"
from 
	accounts
	left join (
		select
			account_id,
			currency,
			sum(value) as value
		from fund_movements
		group by account_id, currency
	) as fund_changes
	on 
		accounts.account_id = fund_changes.account_id
		and accounts.currency = fund_changes.currency
	left join currency_exchanges
	on 
		currency_exchanges.date = date('now')
		and currency_exchanges.currency_to = 'EUR'
		and currency_exchanges.currency_from = accounts.currency
where accounts.initial_balance + fund_changes.value >= 0.01
