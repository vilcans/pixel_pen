//! UI widgets.

use eframe::egui::{Align, Area, Frame, Id, Key, Layout, Order, Response, Ui};

/// Shows a popup if it's open.
/// Based on [`egui::popup::popup_below_widget`], but works also if the popup was opened with a right-click
/// (https://github.com/emilk/egui/issues/198).
/// Also does not limit the width of the popup content.
pub fn popup(
    ui: &Ui,
    popup_id: Id,
    widget_response: &Response,
    add_contents: impl FnOnce(&mut Ui),
) {
    if ui.memory().is_popup_open(popup_id) {
        let parent_clip_rect = ui.clip_rect();

        Area::new(popup_id)
            .order(Order::Foreground)
            .fixed_pos(widget_response.rect.left_bottom())
            .show(ui.ctx(), |ui| {
                ui.set_clip_rect(parent_clip_rect); // for when the combo-box is in a scroll area.
                let frame = Frame::popup(ui.style());
                frame.show(ui, |ui| {
                    ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                        add_contents(ui)
                    });
                });
            });

        if ui.input().key_pressed(Key::Escape)
            || ui.input().pointer.any_click()
                && !widget_response.clicked()
                && !widget_response.secondary_clicked()
        {
            ui.memory().close_popup();
        }
    }
}
