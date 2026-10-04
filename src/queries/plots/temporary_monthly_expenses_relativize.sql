update monthly_expenses_temporary
set value = 100 * monthly_expenses_temporary.value / monthly_expenses_temporary_grouped.value
from (select month, sum(value) as value from monthly_expenses_temporary where value > 0 group by month) as monthly_expenses_temporary_grouped
where monthly_expenses_temporary_grouped.month = monthly_expenses_temporary.month
