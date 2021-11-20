use crate::mutation_monitor::MutationMonitor;
use crate::vic::{self, GlobalColors, VicImage};
use crate::widgets;
use eframe::egui::{self, Color32, Painter, Rect, Sense, Shape, Vec2};
use itertools::Itertools;

pub fn render_palette(
    ui: &mut egui::Ui,
    paint_color: &mut usize,
    image: &mut MutationMonitor<VicImage>,
) {
    ui.horizontal_wrapped(|ui| {
        let interact_size = ui.spacing().interact_size;
        let patch_width = interact_size.x.max(interact_size.y);
        let patch_height = patch_width;
        for color_index in 0..vic::PALETTE_SIZE {
            let color = vic::palette_color(color_index);
            let (patch_rect, response) =
                ui.allocate_exact_size(Vec2::new(patch_width, patch_height), Sense::click());
            palette_patch(
                ui.painter(),
                &patch_rect,
                color.into(),
                color_index == image.colors[GlobalColors::BACKGROUND] as usize,
                color_index == image.colors[GlobalColors::BORDER] as usize,
                color_index == image.colors[GlobalColors::AUX] as usize,
                color_index == *paint_color,
            );
            if response.clicked() {
                *paint_color = color_index;
            }
            let color_description = format!(
                "${0:x} ({0}): {1}",
                color_index,
                vic::palette_entry_name(color_index),
            );
            let popup_id = ui.make_persistent_id(format!("color_popup_{}", color_index));
            if response.secondary_clicked() {
                ui.memory().open_popup(popup_id);
            }
            if !ui.memory().is_popup_open(popup_id) {
                let mut tooltip = color_description.clone();
                let usages = &vic::GLOBAL_COLORS
                    .iter()
                    .filter(|(index, _, _)| image.colors[*index as u32] == color_index as u8)
                    .map(|(_, label, _)| label)
                    .join(", ");
                if !usages.is_empty() {
                    tooltip = format!("{}\n{}", tooltip, usages);
                }
                response.clone().on_hover_text(tooltip);
            }
            widgets::popup(ui, popup_id, &response, |ui| {
                let color_index = color_index as u8;
                ui.label(color_description);
                for (index, label, range) in vic::GLOBAL_COLORS.iter() {
                    let index = *index as u32;
                    if range.contains(&color_index) {
                        let setting = image.colors[index];
                        let mut selected = setting == color_index;
                        if ui.checkbox(&mut selected, *label).clicked() && setting != color_index {
                            println!("Setting {0} to {1}", label, color_index);
                            image.colors[index] = color_index;
                        }
                    }
                }
            });
        }
    });
}

fn palette_patch(
    painter: &Painter,
    rect: &Rect,
    color: Color32,
    selected_background: bool,
    selected_border: bool,
    selected_aux: bool,
    selected_pen: bool,
) {
    let size = rect.width();
    let d = size * 0.2;
    let r = d / 2.0;
    let icon_distance = d * 1.1;
    let number_of_icons = if selected_background { 1 } else { 0 }
        + if selected_border { 1 } else { 0 }
        + if selected_aux { 1 } else { 0 };
    let mut icon_num = 0;
    let mut next_icon_pos = || {
        let i = icon_num;
        icon_num += 1;
        let xoffs = (i as f32 - (number_of_icons - 1) as f32 / 2.0) * icon_distance;
        Rect::from_center_size(rect.center_bottom() + Vec2::new(xoffs, -r), Vec2::new(d, d))
    };

    // The patch
    let patch_rect = Rect::from_min_size(
        rect.left_top() + Vec2::new(0.0, d),
        rect.size() + Vec2::new(0.0, -d * 2.2),
    );
    if selected_pen {
        painter.rect_filled(patch_rect, size * 0.1, color);
    } else {
        painter.rect_filled(patch_rect.shrink(size * 0.05), size * 0.1, color);
    }

    if selected_pen {
        painter.add(Shape::convex_polygon(
            vec![
                patch_rect.center_top(),
                rect.center_top() - Vec2::new(r, 0.0),
                rect.center_top() + Vec2::new(r, 0.0),
            ],
            Color32::WHITE,
            (0.0, Color32::WHITE),
        ));
    }
    if selected_background {
        painter.rect_filled(next_icon_pos(), 0.0, Color32::WHITE);
    }
    if selected_border {
        let width = size * 0.04;
        painter.rect_stroke(
            next_icon_pos().shrink(width / 2.0),
            0.0,
            (width, Color32::WHITE),
        );
    }
    if selected_aux {
        let rect = next_icon_pos();
        painter.circle_filled(rect.center(), rect.width() / 2.0, Color32::WHITE);
    }
}
