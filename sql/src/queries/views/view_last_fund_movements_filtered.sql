select 
	fund_movements.fund_movement_type,
	fund_movements.value,
	fund_movements.currency,
	fund_movements.date,
	fund_movements.party_id,
	accounts.name
from
	fund_movements
	inner join
	accounts
	on fund_movements.account_id = accounts.account_id
where
	accounts.account_id = ?
order by
	fund_movements.date desc,
	fund_movements.party_id desc
limit ?
