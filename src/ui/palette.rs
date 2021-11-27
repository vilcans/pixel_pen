use crate::mutation_monitor::MutationMonitor;
use crate::vic::{self, GlobalColors, PaintColor, VicImage};
use crate::widgets;
use eframe::egui::{self, Color32, Painter, Rect, Sense, Shape, Vec2};
use itertools::Itertools;

const PATCH_CORNER_RADIUS_FRACTION: f32 = 0.1;

pub fn render_palette(
    ui: &mut egui::Ui,
    primary_color: &mut PaintColor,
    secondary_color: &mut PaintColor,
    image: &mut MutationMonitor<VicImage>,
) {
    ui.horizontal_wrapped(|ui| {
        ui.allocate_at_least(
            Vec2::new(0.0, ui.spacing().interact_size.y * 3.0),
            Sense::hover(),
        );
        render_special_color_label(ui, image, PaintColor::Background, "Background", "Can be used in any cell. Click to change.");
        render_patch(ui, image, PaintColor::Background, primary_color, secondary_color);
        render_special_color_label(ui, image, PaintColor::Border, "Border", "Can be used as an additional color in a multicolor cell. Also the color of the screen border. Click to change.");
        render_patch(ui, image, PaintColor::Border, primary_color, secondary_color);
        render_special_color_label(ui, image, PaintColor::Aux, "Aux", "Can be used as an additional color in a multicolor cell. Click to change.");
        render_patch(ui, image, PaintColor::Aux, primary_color, secondary_color);
        ui.separator();
        for color_number in vic::ALLOWED_CHAR_COLORS {
            let patch = PaintColor::CharColor(color_number as u8);
            render_patch(ui, image, patch, primary_color, secondary_color);
        }
    });
}

fn render_patch(
    ui: &mut egui::Ui,
    image: &mut MutationMonitor<VicImage>,
    patch: PaintColor,
    primary_color: &mut PaintColor,
    secondary_color: &mut PaintColor,
) {
    let patch_size = patch_size(ui);
    let (patch_rect, response) = ui.allocate_exact_size(patch_size, Sense::click());
    draw_patch(
        ui.painter(),
        &patch_rect,
        image,
        patch,
        *primary_color,
        *secondary_color,
    );
    if response.clicked() {
        *primary_color = patch;
    }
    if response.secondary_clicked() {
        *secondary_color = patch;
    }
    render_patch_popups(
        image,
        ui,
        response,
        patch,
        *primary_color == patch,
        *secondary_color == patch,
    );
}

/// The clickable label for a special color. Shows a popup if clicked.
fn render_special_color_label(
    ui: &mut egui::Ui,
    image: &mut MutationMonitor<VicImage>,
    patch: PaintColor,
    label: &str,
    tooltip: &str,
) {
    let response = ui.small_button(label);
    let popup_id = ui.make_persistent_id(format!("color_popup_{:?}", patch));
    if !ui.memory().is_popup_open(popup_id) {
        response.clone().on_hover_text(tooltip);
    }
    if response.clicked() {
        ui.memory().open_popup(popup_id);
    }
    render_color_popup(ui, &response, popup_id, image, patch);
}

fn draw_patch(
    painter: &Painter,
    rect: &Rect,
    image: &VicImage,
    patch: PaintColor,
    primary_color: PaintColor,
    secondary_color: PaintColor,
) {
    let rgb = image.true_color_from_paint_color(&patch);

    let size = rect.width();
    let d = size * 0.2;
    let r = d / 2.0;

    // The patch
    let patch_rect = Rect::from_min_size(
        rect.left_top() + Vec2::new(0.0, d),
        rect.size() + Vec2::new(0.0, -d * 2.2),
    );
    let corner = size * PATCH_CORNER_RADIUS_FRACTION;
    if primary_color == patch {
        painter.rect_filled(patch_rect, corner, rgb);
    } else {
        painter.rect_filled(patch_rect.shrink(size * 0.05), corner, rgb);
    }

    // If primary and secondary are the same, make room for both
    let offset = if primary_color == secondary_color {
        Vec2::new(r * 1.5, 0.0)
    } else {
        Vec2::new(0.0, 0.0)
    };
    if primary_color == patch {
        painter.add(Shape::convex_polygon(
            vec![
                patch_rect.center_top() - offset,
                rect.center_top() - offset - Vec2::new(r, 0.0),
                rect.center_top() - offset + Vec2::new(r, 0.0),
            ],
            Color32::WHITE,
            (1.0, Color32::WHITE),
        ));
    }
    if secondary_color == patch {
        painter.add(Shape::convex_polygon(
            vec![
                patch_rect.center_top() + offset,
                rect.center_top() + offset - Vec2::new(r, 0.0),
                rect.center_top() + offset + Vec2::new(r, 0.0),
            ],
            Color32::WHITE,
            (1.0, Color32::BLACK),
        ));
    }
}

fn render_patch_popups(
    image: &mut MutationMonitor<VicImage>,
    _ui: &mut egui::Ui,
    response: egui::Response,
    patch: PaintColor,
    selected_as_primary: bool,
    selected_as_secondary: bool,
) {
    let color_description = match patch {
        PaintColor::Background => format!(
            "Background ({})",
            vic::palette_entry_name(image.colors[GlobalColors::BACKGROUND])
        ),
        PaintColor::Border => format!(
            "Border ({})",
            vic::palette_entry_name(image.colors[GlobalColors::BORDER])
        ),
        PaintColor::Aux => format!(
            "Auxiliary ({})",
            vic::palette_entry_name(image.colors[GlobalColors::AUX])
        ),
        PaintColor::CharColor(index) => format!(
            "Character color {}: {}",
            index,
            vic::palette_entry_name(index)
        ),
    };
    let selected_text = match (selected_as_primary, selected_as_secondary) {
        (false, false) => "Left/right click to select as primary/secondary",
        (true, false) => "Selected primary color",
        (false, true) => "Selected secondary color",
        (true, true) => "Selected primary and secondary color",
    };
    response.on_hover_text(format!("{}\n{}", color_description, selected_text));
}

fn render_color_popup(
    ui: &mut egui::Ui,
    response: &egui::Response,
    popup_id: egui::Id,
    image: &mut MutationMonitor<VicImage>,
    patch: PaintColor,
) {
    widgets::popup(ui, popup_id, response, |ui| {
        let patch_size = patch_size(ui);
        for (_, indices) in patch.selectable_colors().group_by(|i| i / 8).into_iter() {
            ui.horizontal(|ui| {
                for index in indices {
                    let index = index as u8;
                    let label = vic::palette_entry_name(index);
                    let (patch_rect, response) = ui.allocate_exact_size(patch_size, Sense::click());
                    ui.painter().rect_filled(
                        patch_rect,
                        patch_rect.size().y * PATCH_CORNER_RADIUS_FRACTION,
                        vic::palette_color(index),
                    );
                    response.clone().on_hover_text(label);
                    if response.clicked() {
                        match patch {
                            PaintColor::Background => {
                                image.colors[GlobalColors::BACKGROUND] = index
                            }
                            PaintColor::Border => image.colors[GlobalColors::BORDER] = index,
                            PaintColor::Aux => image.colors[GlobalColors::AUX] = index,
                            PaintColor::CharColor(_) => {}
                        }
                    }
                }
            });
        }
    });
}

fn patch_size(ui: &egui::Ui) -> Vec2 {
    let interact_size = ui.spacing().interact_size;
    let patch_width = interact_size.x.max(interact_size.y);
    let patch_height = patch_width;
    Vec2::new(patch_width, patch_height)
}
