use crate::actions::{Action, DocAction};
use crate::mutation_monitor::MutationMonitor;
use crate::vic::{self, PixelColor, Register, VicImage, VicPalette};
use crate::widgets;
use eframe::egui::{self, Color32, Painter, Rect, Sense, Shape, Vec2};
use itertools::Itertools;

const PATCH_CORNER_RADIUS_FRACTION: f32 = 0.1;

pub fn render_palette(
    ui: &mut egui::Ui,
    primary_color: &mut PixelColor,
    secondary_color: &mut PixelColor,
    image: &mut MutationMonitor<VicImage>,
) -> Option<Action> {
    let mut action = None;

    let allocate = Vec2::new(0.0, ui.spacing().interact_size.y * 2.5);
    ui.horizontal_wrapped(|ui| {
        ui.vertical(|ui| {
            ui.small("Color Registers").on_hover_text("Global color settings that affect the whole screen.");
            ui.horizontal(|ui| {
                ui.allocate_at_least(
                    allocate,
                    Sense::hover(),
                );
                for (patch, label, tooltip) in [
                    (PixelColor::Background, "Background", "Can be used in any cell. Click to change."),
                    (PixelColor::Border, "Border", "Can be used as an additional color in a multicolor cell. Also the color of the screen border. Click to change."),
                    (PixelColor::Aux, "Aux", "Can be used as an additional color in a multicolor cell. Click to change."),
                ] {
                    if let Some(a) = render_special_color_label(ui, patch, label, tooltip){
                        action = Some(a);
                    }
                    render_patch(ui, image, patch, primary_color, secondary_color);
                }
            });
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.small("Character Colors").on_hover_text("A color that can be set for an individual character cell.");
            ui.horizontal(|ui| {
                ui.allocate_at_least(
                    allocate,
                    Sense::hover(),
                );
                for color_number in vic::ALLOWED_CHAR_COLORS {
                    let patch = PixelColor::CharColor(color_number as u8);
                    render_patch(ui, image, patch, primary_color, secondary_color);
                }
            });
        });
    });
    action
}

fn render_patch(
    ui: &mut egui::Ui,
    image: &mut MutationMonitor<VicImage>,
    patch: PixelColor,
    primary_color: &mut PixelColor,
    secondary_color: &mut PixelColor,
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
        if *secondary_color == patch {
            *secondary_color = *primary_color;
        }
        *primary_color = patch;
    }
    if response.secondary_clicked() {
        if *primary_color == patch {
            *primary_color = *secondary_color;
        }
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
    patch: PixelColor,
    label: &str,
    tooltip: &str,
) -> Option<Action> {
    let response = ui.small_button(label);
    let popup_id = ui.make_persistent_id(format!("color_popup_{:?}", patch));
    if !ui.memory().is_popup_open(popup_id) {
        response.clone().on_hover_text(tooltip);
    }
    if response.clicked() {
        ui.memory().open_popup(popup_id);
    }
    render_color_popup(ui, &response, popup_id, patch)
}

fn draw_patch(
    painter: &Painter,
    rect: &Rect,
    image: &VicImage,
    patch: PixelColor,
    primary_color: PixelColor,
    secondary_color: PixelColor,
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
            Color32::BLACK,
            (1.0, Color32::WHITE),
        ));
    }
}

fn render_patch_popups(
    image: &mut MutationMonitor<VicImage>,
    _ui: &mut egui::Ui,
    response: egui::Response,
    patch: PixelColor,
    selected_as_primary: bool,
    selected_as_secondary: bool,
) {
    let color_description = match patch {
        PixelColor::Background => format!(
            "Background ({})",
            VicPalette::name(image.global_colors().background)
        ),
        PixelColor::Border => format!(
            "Border ({})",
            VicPalette::name(image.global_colors().border)
        ),
        PixelColor::Aux => format!(
            "Auxiliary ({})",
            VicPalette::name(image.global_colors().aux)
        ),
        PixelColor::CharColor(index) => {
            format!("Character color {}: {}", index, VicPalette::name(index))
        }
    };
    let selected_text = match (selected_as_primary, selected_as_secondary) {
        (false, false) => "Left/right click to select as primary/secondary.",
        (true, false) => "Selected primary color. Right-click to swap with secondary color.",
        (false, true) => "Selected secondary color. Click to swap with primary color.",
        (true, true) => "Selected primary and secondary color.",
    };
    response.on_hover_text(format!("{}\n{}", color_description, selected_text));
}

fn render_color_popup(
    ui: &mut egui::Ui,
    response: &egui::Response,
    popup_id: egui::Id,
    patch: PixelColor,
) -> Option<Action> {
    let mut action = None;
    widgets::popup(ui, popup_id, response, |ui| {
        let patch_size = patch_size(ui);
        for (_, indices) in patch.selectable_colors().group_by(|i| i / 8).into_iter() {
            ui.horizontal(|ui| {
                for index in indices {
                    let index = index as u8;
                    let label = VicPalette::name(index);
                    let (patch_rect, response) = ui.allocate_exact_size(patch_size, Sense::click());
                    ui.painter().rect_filled(
                        patch_rect,
                        patch_rect.size().y * PATCH_CORNER_RADIUS_FRACTION,
                        VicPalette::color(index),
                    );
                    response.clone().on_hover_text(label);
                    if response.clicked() {
                        match patch {
                            PixelColor::Background => {
                                action = Some(Action::Document(DocAction::ChangeRegister {
                                    index: Register::Background,
                                    value: index,
                                }))
                            }
                            PixelColor::Border => {
                                action = Some(Action::Document(DocAction::ChangeRegister {
                                    index: Register::Border,
                                    value: index,
                                }))
                            }
                            PixelColor::Aux => {
                                action = Some(Action::Document(DocAction::ChangeRegister {
                                    index: Register::Aux,
                                    value: index,
                                }))
                            }
                            PixelColor::CharColor(_) => {}
                        }
                    }
                }
            });
        }
    });
    action
}

fn patch_size(ui: &egui::Ui) -> Vec2 {
    let interact_size = ui.spacing().interact_size;
    let patch_width = interact_size.x.max(interact_size.y);
    let patch_height = patch_width;
    Vec2::new(patch_width, patch_height)
}
