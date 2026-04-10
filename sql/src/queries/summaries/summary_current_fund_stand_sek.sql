select
	accounts.name,
	accounts.country,
	'SEK' as currency,
	accounts.account_type,
	(accounts.initial_balance + fund_changes.value) 
		* (case when currency_exchanges_to_eur.value is null then 1.0 else currency_exchanges_to_eur.value end) 
		* (case when currency_exchanges_to_sek.value is null then 1.0 else currency_exchanges_to_sek.value end) 
		as "current_value!"
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
	left join currency_exchanges as currency_exchanges_to_eur
	on 
		currency_exchanges_to_eur.date = date('now')
		and currency_exchanges_to_eur.currency_to = 'EUR'
		and currency_exchanges_to_eur.currency_from = accounts.currency
	left join currency_exchanges as currency_exchanges_to_sek
	on
		currency_exchanges_to_sek.date = date('now')
		and currency_exchanges_to_sek.currency_to = 'SEK'
		and currency_exchanges_to_sek.currency_from = currency_exchanges_to_eur.currency_to	
where accounts.initial_balance + fund_changes.value >= 0.01
