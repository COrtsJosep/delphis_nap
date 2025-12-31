use crate::modules::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use polars::prelude::PolarsError;

impl AppState {
    pub fn handle_show_error_window(&mut self, ctx: &egui::Context) -> () {
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("error_window"),
            egui::ViewportBuilder::default()
                .with_title("An error ocurred!")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default().show(ctx, |ui| ui.label(self.error_message.clone()));
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_error_window = false;
                }
            },
        );
    }

    pub fn throw_error(&mut self, error: Box<dyn std::error::Error>) -> () {
        self.error_message = error.to_string();
        self.show_error_window = true;
    }

    pub fn throw_polars_error(&mut self, error: PolarsError) -> () {
        self.error_message = error.to_string();
        self.show_error_window = true;
    }
}
