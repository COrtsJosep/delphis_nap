use crate::modules::database::plotter::BarplotType;
use crate::modules::financial::*;
use crate::modules::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use eframe::egui::ComboBox;
use egui_extras::*;
use strum::IntoEnumIterator;

impl AppState {
    pub fn handle_show_fund_evolution_plot(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("fund_evolution_plot_window"),
            egui::ViewportBuilder::default()
                .with_title("Fund evolution plot window")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::remainder().at_least(120.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                egui::Grid::new("fund_evolution_plot")
                                    .num_columns(3)
                                    .spacing([45.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Currency:").on_hover_text(
                                            "Currency in which to express the ammounts.",
                                        );
                                        ComboBox::from_id_salt("Fund evolution plot currency")
                                            .selected_text(format!(
                                                "{}",
                                                self.fund_evolution_plot_currency
                                            ))
                                            .show_ui(ui, |ui| {
                                                for possible_fund_evolution_plot_currency in
                                                    Currency::iter()
                                                {
                                                    ui.selectable_value(
                                            &mut self.fund_evolution_plot_currency,
                                            possible_fund_evolution_plot_currency.clone(),
                                            format!("{possible_fund_evolution_plot_currency}"),
                                        );
                                                }
                                            });
                                        ui.end_row();

                                        ui.label("");
                                        if ui.button("Generate!").clicked() {
                                            match self
                                                .database
                                                .funds_evolution(&self.fund_evolution_plot_currency)
                                            {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    self.throw_error(e);
                                                }
                                            }

                                            // forget the old one
                                            ui.ctx().forget_all_images();
                                        }
                                    });
                                ui.separator();
                            });
                            strip.cell(|ui| {
                                ui.image("file://figures/funds_evolution.svg");
                                ui.separator();
                            });
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_fund_evolution_plot_window = false;
                }
            },
        );
    }

    pub fn handle_show_expense_category_plot(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("expense_category_plot_window"),
            egui::ViewportBuilder::default()
                .with_title("Expense category plot window")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| {
                    StripBuilder::new(ui)
                        .size(Size::exact(40.0))
                        .size(Size::remainder().at_least(120.0))
                        .vertical(|mut strip| {
                            strip.cell(|ui| {
                                egui::Grid::new("expense_category_plot")
                                    .num_columns(3)
                                    .spacing([45.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Currency:").on_hover_text(
                                            "Currency in which to express the ammounts.",
                                        );
                                        ComboBox::from_id_salt("Expense category plot currency")
                                            .selected_text(format!(
                                                "{}",
                                                self.expense_category_plot_currency
                                            ))
                                            .show_ui(ui, |ui| {
                                                for possible_expense_category_plot_currency in
                                                    Currency::iter()
                                                {
                                                    ui.selectable_value(
                                            &mut self.expense_category_plot_currency,
                                            possible_expense_category_plot_currency.clone(),
                                            format!("{possible_expense_category_plot_currency}"),
                                        );
                                                }
                                            });
                                        ui.end_row();

                                        ui.label("Barplot Type:").on_hover_text(
                                            "Absolute: Column height is the expense ammount. Relative: Column height is normalized to 100%.",
                                        );
                                        ComboBox::from_id_salt("Expense category plot type")
                                            .selected_text(format!(
                                                "{}",
                                                self.expense_category_plot_type
                                            ))
                                            .show_ui(ui, |ui| {
                                                for possible_expense_category_plot_type in
                                                    BarplotType::iter()
                                                {
                                                    ui.selectable_value(
                                            &mut self.expense_category_plot_type,
                                            possible_expense_category_plot_type.clone(),
                                            format!("{possible_expense_category_plot_type}"),
                                        );
                                                }
                                            });
                                        ui.end_row();

                                        ui.label("");
                                        if ui.button("Generate!").clicked() {
                                            match self.database.monthly_expenses(
                                                &self.expense_category_plot_currency,
                                                &self.expense_category_plot_type,
                                            ) {
                                                Ok(_) => {},
                                                Err(e) => {self.throw_error(e);}
                                            }

                                            ui.ctx().forget_all_images();
                                        }
                                    });
                                ui.separator();
                            });
                            strip.cell(|ui| {
                                ui.image("file://figures/monthly_expenses.svg");
                                ui.separator();
                            });
                        });
                });
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_expense_category_plot_window = false;
                }
            },
        );
    }
}
