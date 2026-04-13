use crate::financial::Currency;
use crate::financial_database::DATE_FORMAT;
use crate::FinancialDataBase;
use chrono::NaiveDate;
use plotters::prelude::*;
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
}
