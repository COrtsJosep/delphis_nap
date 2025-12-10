use crate::modules::currency_exchange::CurrencyExchange;
use crate::modules::database::palettes::fetch_palette;
use crate::modules::database::DataBase;
use crate::modules::financial::Currency;
use chrono::{Months, NaiveDate};
use plotters::prelude::*;
use polars::prelude::*;
use std::fmt::Display;
use std::fs::{create_dir, File};
use std::path::Path;
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

/// Returns the earliest / latest date in the dataframe's "date" column.
/// Panics if no such column.
fn extreme_date(data_frame: &DataFrame, extrema: Extrema) -> NaiveDate {
    let i: usize = match extrema {
        Extrema::MIN => 0,
        Extrema::MAX => data_frame.height() - 1,
    };
    data_frame
        .sort(["date"], Default::default())
        .unwrap()
        .column("date")
        .unwrap()
        .date()
        .unwrap()
        .as_date_iter()
        .collect::<Vec<Option<NaiveDate>>>()[i]
        .unwrap()
}

/// Returns the smallest / largest value in the dataframe's "value" column.
/// Panics if no such column.
fn extreme_value(data_frame: &DataFrame, extrema: Extrema) -> f64 {
    let lazy_frame = match extrema {
        Extrema::MIN => data_frame
            .clone()
            .lazy()
            .filter(col("value").lt(0.0))
            .group_by(["date"])
            .agg([sum("value")])
            .min(),
        Extrema::MAX => data_frame
            .clone()
            .lazy()
            .filter(col("value").gt_eq(0.0))
            .group_by(["date"])
            .agg([sum("value")])
            .max(),
    };

    let extreme_value = lazy_frame
        .collect()
        .unwrap()
        .column("value")
        .unwrap()
        .f64()
        .unwrap()
        .get(0)
        .unwrap_or(0.0);

    match extrema {
        Extrema::MIN => {
            if extreme_value > 0.0 {
                0.0
            } else {
                extreme_value
            }
        }
        Extrema::MAX => extreme_value,
    }
}

impl DataBase {
    // Writes a funds evolution plot (and optionally a csv too), with x-axis
    // date, and y-axis total funds.
    pub(crate) fn funds_evolution(&self, currency_to: &Currency) -> () {
        let currency_exchange: CurrencyExchange = CurrencyExchange::init();

        // Fetch the ammounts in the different accounts in the date
        // of their creation.
        let initial_balances: DataFrame = self
            .account_table
            .data_frame
            .clone()
            .lazy()
            .select([
                col("initial_balance").alias("value"),
                col("currency"),
                col("creation_date").alias("date"),
            ])
            .collect()
            .expect("Failed to select account table");

        // Fetch the table with fund movements.
        let mut funds_table: DataFrame = self
            .funds_table
            .data_frame
            .clone()
            .select(["value", "currency", "date"])
            .expect("Failed to select funds table");

        // First step is getting all fund changes in history, and to those, adding the initial
        // balances of all accounts.
        funds_table = funds_table
            .vstack(&initial_balances)
            .expect("Could not append new data");

        // Next step is converting values into the same currency.
        funds_table = currency_exchange.exchange_currencies(currency_to, funds_table);

        // Final data manipulation step involves grouping fund changes per natural
        // day, expanding to all days without movements, and doing the cumsum!
        let mut result: DataFrame = funds_table
            .lazy()
            .sort(["date"], Default::default())
            .group_by_dynamic(
                col("date"),
                [],
                DynamicGroupOptions {
                    every: Duration::parse("1d"),
                    period: Duration::parse("1d"),
                    offset: Duration::parse("0"),
                    ..Default::default()
                },
            )
            .agg([col("value").sum()])
            .collect()
            .expect("Failed to aggregate by day")
            .upsample::<[String; 0]>([], "date", Duration::parse("1d"))
            .expect("Failed to expand date")
            .fill_null(FillNullStrategy::Zero)
            .expect("Failed to fill null values")
            .lazy()
            .select([
                col("date").alias("date"),
                col("value").cum_sum(false).alias("value"),
            ])
            .collect()
            .expect("Failed to cumsum");

        if currency_to == &Currency::EUR {
            // I like having the data in csv
            let file_name = "data/funds_evolution_table.csv";
            let path: &Path = Path::new(file_name);
            if !path.parent().expect("path does not have parent").exists() {
                let _ = create_dir(path.parent().expect("path does not have parents"));
            }

            let mut file =
                File::create(path).expect("Could not create file funds_evolution_table.csv");

            CsvWriter::new(&mut file)
                .include_header(true)
                .with_separator(b',')
                .finish(&mut result)
                .expect("Failed to save fund evolution table.");
        }

        // Now comes the plotting part. First extract data as vectors.
        let dates: Vec<NaiveDate> = result
            .column("date")
            .expect("Could not find date column")
            .date()
            .expect("Could not convert date column to date (what the hell does that mean?)")
            .as_date_iter()
            .map(|opt_date| opt_date.expect("Found null value in date column"))
            .collect::<Vec<NaiveDate>>();

        let values: Vec<f64> = result
            .column("value")
            .expect("Could not find value column")
            .f64()
            .expect("Could not convert date column to f64 (what the hell does that mean?)")
            .into_no_null_iter()
            .collect();

        let bankrupcy_values: Vec<f64> = vec![0.0; values.len()];

        // Then create the plot
        let root = SVGBackend::new("figures/funds_evolution.svg", (800, 640)).into_drawing_area();
        root.fill(&WHITE).expect("Failed to fill plotting root");

        let mut chart = ChartBuilder::on(&root)
            .caption("Evolution of Total Funds", ("sans-serif", 20).into_font())
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(
                dates[0]..dates[dates.len() - 1],
                0.0..values.iter().cloned().fold(0. / 0., f64::max),
            )
            .expect("Failed to build chart");

        chart
            .configure_mesh()
            .x_desc("Time")
            .x_label_style(("sans-serif", 15).into_font())
            .y_desc(currency_to.to_string().as_str())
            .y_label_formatter(&|y| format!("{:.0}", *y))
            .y_label_style(("sans-serif", 15).into_font())
            .draw()
            .expect("Failed to draw");

        chart
            .draw_series(LineSeries::new(
                dates.iter().zip(values.iter()).map(|(d, v)| (*d, *v)),
                &BLACK,
            ))
            .expect("Failed to draw line")
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
            .expect("Failed to draw line")
            .label("Bankrupcy")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();

        // Finally save the plot
        root.present().expect("Failed to present plot");
    }

    // Creates a stacked barplot of monthly expenses. One column per month, split into
    // expense categories.
    pub(crate) fn monthly_expenses(
        &self,
        currency_to: &Currency,
        barplot_type: &BarplotType,
    ) -> () {
        let currency_exchange: CurrencyExchange = CurrencyExchange::init();

        let mut data_frame: DataFrame = self.expenses_table.data_frame.clone();

        // First: convert the ammounts to the desired output currency,
        // and group by month.
        data_frame = currency_exchange
            .exchange_currencies(currency_to, data_frame)
            .lazy()
            .sort(["date"], Default::default())
            .group_by_dynamic(
                col("date"),
                [col("category")],
                DynamicGroupOptions {
                    every: Duration::parse("1mo"),
                    period: Duration::parse("1mo"),
                    offset: Duration::parse("0"),
                    ..Default::default()
                },
            )
            .agg([col("value").sum()])
            .collect()
            .expect("Failed to aggregate by month");

        // Then: if the column plot is relative (columns add to 100),
        // normalize all months to add to 100.
        let data_frame = match barplot_type {
            BarplotType::ABSOLUTE => data_frame,
            BarplotType::RELATIVE => {
                let totals_data_frame = data_frame
                    .clone()
                    .lazy()
                    .filter(col("value").gt_eq(0.0))
                    .group_by(["date"])
                    .agg([sum("value")])
                    .with_column(col("value").alias("total"))
                    .select([col("date"), col("total")]);

                data_frame
                    .clone()
                    .lazy()
                    .left_join(totals_data_frame, col("date"), col("date"))
                    .with_column((lit(100.0) * col("value") / col("total")).alias("value"))
                    .collect()
                    .unwrap()
            }
        };

        // Get vector of unique months.
        let unique_months: Vec<NaiveDate> = data_frame
            .column("date")
            .unwrap()
            .unique_stable()
            .unwrap()
            .date()
            .unwrap()
            .as_date_iter()
            .map(|date| date.unwrap())
            .collect::<Vec<NaiveDate>>();

        // Now get vector of unique categories, sorted from categories
        // with the largest to smallest spending.
        let binding = data_frame
            .sort(
                ["value"],
                SortMultipleOptions::new().with_order_descending(true),
            )
            .unwrap()
            .column("category")
            .unwrap()
            .unique_stable()
            .unwrap();

        let unique_categories: Vec<&str> = binding
            .str()
            .unwrap()
            .iter()
            .map(|category| category.unwrap())
            .collect::<Vec<&str>>();
        let num_categories: usize = unique_categories.len();

        // Initialize the plot.
        let root = SVGBackend::new("figures/monthly_expenses.svg", (800, 640)).into_drawing_area();
        root.fill(&WHITE).expect("Failed to set chart background");

        // Initialize axis, etc.
        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Expenses by Month and Category",
                ("sans-serif", 20).into_font(),
            )
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(
                extreme_date(&data_frame, Extrema::MIN)
                    ..extreme_date(&data_frame, Extrema::MAX)
                        .checked_add_months(Months::new(2))
                        .unwrap(),
                ((extreme_value(&data_frame, Extrema::MIN) - 0.001) * 1.05)
                    ..(extreme_value(&data_frame, Extrema::MAX) * 1.05),
            )
            .expect("Failed to set chart axis");

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

        mesh.x_desc("Date")
            .x_label_style(("sans-serif", 15).into_font())
            .y_label_style(("sans-serif", 15).into_font())
            .draw()
            .expect("Failed to render mesh");

        // fetch the colour palette
        let palette: Vec<RGBAColor> = fetch_palette(num_categories);
        // Plot the columns
        for (index_m, month) in unique_months.iter().enumerate() {
            let mut y0_pos: f64 = 0.0;
            let mut y0_neg: f64 = 0.0;
            for (mut index_c, category) in unique_categories.iter().enumerate() {
                index_c = index_c % 14; // wrap around the maximum number of colours
                let colour = palette[index_c];
                let x0 = *month;
                let x1 = month.checked_add_months(Months::new(1)).unwrap();
                let height = data_frame
                    .clone()
                    .lazy()
                    .filter(
                        col("category")
                            .eq(lit(*category))
                            .and(col("date").eq(lit(*month))),
                    )
                    .collect()
                    .unwrap()
                    .column("value")
                    .unwrap()
                    .f64()
                    .unwrap()
                    .max() // easiest way to get the only value, if exists
                    .unwrap_or(0.0);

                let y0 = if height > 0.0 { y0_pos } else { y0_neg };
                let y1 = y0 + height;

                let mut bar = Rectangle::new([(x0, y0), (x1, y1)], colour.filled());
                bar.set_margin(0, 0, 5, 5);

                let ctx = chart.draw_series(vec![bar]).expect("Failed");
                if index_m == 0 {
                    let style = colour.stroke_width(10);
                    ctx.label(*category).legend(move |(x, y)| {
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
    }
}
