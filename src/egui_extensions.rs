use eframe::egui::{Response, Ui};

pub trait EnhancedResponse {
    fn clicked_with_close(&self, ui: &mut Ui) -> bool;
}
impl EnhancedResponse for Response {
    fn clicked_with_close(&self, ui: &mut Ui) -> bool {
        self.clicked() && {
            ui.close_menu();
            true
        }
    }
}
