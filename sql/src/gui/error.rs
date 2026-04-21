use crate::gui::{AppState, WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use egui::{Ui};

impl AppState {
    pub fn handle_show_error_window(&mut self, ui: &mut Ui) -> () {
        ui.ctx().show_viewport_immediate(
            egui::ViewportId::from_hash_of("error_window"),
            egui::ViewportBuilder::default()
                .with_title("An error ocurred!")
                .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
            |ui, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default()
                    .show_inside(ui, |ui| ui.label(self.error_message.clone()));
                if ui.input(|i| i.viewport().close_requested()) {
                    self.show_error_window = false;
                }
            },
        );
    }

    pub fn throw_sqlx_error(&mut self, error: sqlx::Error) -> () {
        self.error_message = error.to_string();
        self.show_error_window = true;
    }
}
