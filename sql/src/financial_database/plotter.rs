use crate::financial::Currency;
use crate::financial_database::palettes::fetch_palette;
use crate::financial_database::DATE_FORMAT;
use crate::FinancialDataBase;
use chrono::NaiveDate;
use plotters::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;
use strum_macros::EnumIter;

enum Extrema {
    MIN,
    MAX,
}

#[derive(EnumIter, Eq, PartialEq)]
pub(crate) enum BarplotType {
    RELATIVE,
    ABSOLUTE,
}

impl Default for BarplotType {
    fn default() -> Self {
        BarplotType::ABSOLUTE
    }
}

impl Display for BarplotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BarplotType::ABSOLUTE => "Absolute".to_string(),
            BarplotType::RELATIVE => "Relative".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl BarplotType {
    pub(crate) fn clone(&self) -> BarplotType {
        match self {
            BarplotType::ABSOLUTE => BarplotType::ABSOLUTE,
            BarplotType::RELATIVE => BarplotType::RELATIVE,
        }
    }
}

impl FinancialDataBase {
    // Writes a funds evolution plot, with x-axis
    // date, and y-axis total funds.
    pub(crate) async fn funds_evolution(
        &mut self,
        currency_to: &Currency,
    ) -> Result<(), sqlx::Error> {
        let currency_to_string: String = currency_to.to_string();

        // First fetch the values
        let records = sqlx::query_file!(
            "src/queries/plots/plot_funds_evolution.sql",
            currency_to_string
        )
        .fetch_all(&mut self.connection)
        .await?;

        let mut dates: Vec<NaiveDate> = vec![];
        let mut fund_values: Vec<f64> = vec![];
        let mut bankrupcy_values: Vec<f64> = vec![];
        for record in records {
            dates.push(NaiveDate::parse_from_str(record.date.as_str(), DATE_FORMAT).unwrap());
            fund_values.push(record.value);
            bankrupcy_values.push(0.0);
        }

        // Then create the plot
        let root = SVGBackend::new("figures/funds_evolution.svg", (800, 640)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption("Evolution of Total Funds", ("sans-serif", 20).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(
                dates[0]..dates[dates.len() - 1],
                0.0..fund_values.iter().cloned().fold(0. / 0., f64::max),
            )
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Time")
            .x_label_style(("sans-serif", 15).into_font())
            .y_desc(currency_to.to_string().as_str())
            .y_label_formatter(&|y| format!("{:.0}", *y))
            .y_label_style(("sans-serif", 15).into_font())
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                dates.iter().zip(fund_values.iter()).map(|(d, v)| (*d, *v)),
                &BLACK,
            ))
            .unwrap()
            .label("Total Funds")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));

        chart
            .draw_series(LineSeries::new(
                dates
                    .iter()
                    .zip(bankrupcy_values.iter())
                    .map(|(d, v)| (*d, *v)),
                &RED,
            ))
            .unwrap()
            .label("Bankrupcy")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();

        // Finally save the plot
        root.present().unwrap();

        Ok(())
    }

    // Creates a stacked barplot of monthly expenses. One column per month, split into
    // expense categories.
    pub(crate) async fn monthly_expenses(
        &mut self,
        currency_to: &Currency,
        barplot_type: &BarplotType,
    ) -> Result<(), sqlx::Error> {
        let currency_to_string: String = currency_to.to_string();

        let records = match barplot_type {
            BarplotType::ABSOLUTE => {
                sqlx::query_file_as!(
                    MonthlyExpensesRecord,
                    "src/queries/plots/plot_monthly_expenses_absolute.sql",
                    currency_to_string
                )
                .fetch_all(&mut self.connection)
                .await?
            }
            BarplotType::RELATIVE => {
                sqlx::query_file_as!(
                    MonthlyExpensesRecord,
                    "src/queries/plots/plot_monthly_expenses_relative.sql",
                    currency_to_string
                )
                .fetch_all(&mut self.connection)
                .await?
            }
        };

        let mut months_hm: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let mut unique_categories: Vec<String> = vec![];
        for record in records {
            let category_hm = months_hm.entry(record.date).or_insert_with(HashMap::new);
            category_hm.insert(record.category.clone(), record.aggregate_value);
            if !unique_categories.contains(&record.category) {
                unique_categories.push(record.category);
            }
        }

        let mut unique_months: Vec<String> = months_hm.keys().map(|m| m.to_owned()).collect();
        unique_months.sort_unstable();

        // now we have:
        // - a vec of unique months, sorted asc
        // - a vec of unique categories, sorted desc on largest expenses per month
        // - a dict of dicts (one per month) of string (category) -> f64 (agg value)
        //
        // so we can start plotting!

        // Initialize the plot.
        let root = SVGBackend::new("figures/monthly_expenses.svg", (800, 640)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        // Initialize axis, etc.
        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Expenses by Month and Category",
                ("sans-serif", 20).into_font(),
            )
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(unique_months.into_segmented(), 0.0..1.0) // TODO
            .unwrap();

        // Initialize the plotted objects.
        let mut mesh = chart.configure_mesh();
        mesh.disable_x_mesh().light_line_style(WHITE);

        // Set the correct y-axis labels depending on the plot type.
        match barplot_type {
            BarplotType::ABSOLUTE => {
                mesh.y_label_formatter(&|x| format!("{:.0}", x))
                    .y_desc(currency_to.to_string().as_str());
            }
            BarplotType::RELATIVE => {
                mesh.y_label_formatter(&|x| format!("{:.0}%", x))
                    .y_desc("Percentage of Total Expenses");
            }
        };

        mesh.x_desc("Month")
            .x_label_style(("sans-serif", 15).into_font())
            .y_label_style(("sans-serif", 15).into_font())
            .draw()
            .unwrap();

        // fetch the colour palette
        let palette: Vec<RGBAColor> = fetch_palette(unique_categories.len());
        // Plot the columns
        for (index_m, month) in unique_months.iter().enumerate() {
            let mut y0_pos: f64 = 0.0;
            let mut y0_neg: f64 = 0.0;
            for (mut index_c, category) in unique_categories.iter().enumerate() {
                index_c = index_c % 14; // wrap around the maximum number of colours
                let colour = palette[index_c];
                let x0 = SegmentValue::CenterOf(month);
                let height = match months_hm
                    .get(month)
                    .expect("The month cannot not be there, by construction.")
                    .get(category)
                {
                    Some(value) => *value,
                    None => 0.0, // no expenses of that category in the given month
                };

                let y0 = if height > 0.0 { y0_pos } else { y0_neg };
                let y1 = y0 + height;

                let mut bar = Rectangle::new([(x0.clone(), y0), (x0, y1)], colour.filled());
                bar.set_margin(0, 0, 5, 5);

                let ctx = chart.draw_series(vec![bar]).unwrap();
                if index_m == 0 {
                    let style = colour.stroke_width(10);
                    ctx.label(category).legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], style.clone())
                    });
                }

                if height > 0.0 {
                    y0_pos = y1;
                } else {
                    y0_neg = y1;
                }
            }
        }

        // Finally, create the legend and export.
        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();

        Ok(())
    }
}

struct MonthlyExpensesRecord {
    date: String,
    category: String,
    aggregate_value: f64,
}
