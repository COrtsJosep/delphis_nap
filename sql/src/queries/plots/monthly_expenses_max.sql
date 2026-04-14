select max(month_total) as "max!"
from (
    select sum(value) as month_total
    from monthly_expenses_temporary
	where value > 0.0
    group by month
)
