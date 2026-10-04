select
	accounts.name,
	accounts.country,
	accounts.currency,
	accounts.account_type,
	accounts.initial_balance + fund_changes.value as current_value
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
where accounts.initial_balance + fund_changes.value >= 0.01
