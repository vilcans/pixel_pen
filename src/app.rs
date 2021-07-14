use eframe::{
    egui::{
        self, paint::Mesh, Align2, Color32, Painter, Pos2, Rect, Response, Sense, Shape, TextStyle,
        TextureId, Vec2,
    },
    epi::{self, TextureAllocator},
};
use imgref::ImgVec;
use itertools::Itertools;

use crate::{
    coords::{PixelTransform, Point},
    document::Document,
    mutation_monitor::MutationMonitor,
    scaling, storage,
    system::SystemFunctions,
    ui,
    vic::{self, GlobalColors, VicImage},
    widgets,
};

// Don't scale the texture more than this to avoid huge textures when zooming.
const MAX_SCALE: u32 = 8;

#[derive(PartialEq, Debug)]
enum Mode {
    Import,
    PixelPaint,
    ColorPaint,
}

struct Texture {
    pub id: TextureId,
    pub width: usize,
    pub height: usize,
}

struct UiState {
    mode: Mode,
    zoom: f32,
}

pub struct Application {
    doc: Document,
    ui_state: UiState,
    image_texture: Option<Texture>,
    pub system: SystemFunctions,
}

impl Default for Application {
    fn default() -> Self {
        Self::with_doc(Document::default())
    }
}

impl epi::App for Application {
    fn name(&self) -> &str {
        "Pixel Pen"
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Application {
            ui_state,
            doc,
            image_texture,
            system,
        } = self;
        let (width, height) = doc.image.pixel_size();
        let mut new_doc = None;

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // Menu bar
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if system.open_file_dialog.is_some() && ui.button("Open...").clicked() {
                        match system.open_file_dialog() {
                            Ok(Some(filename)) => {
                                match storage::load_any_file(std::path::Path::new(&filename)) {
                                    Ok(doc) => {
                                        new_doc = Some(doc);
                                    }
                                    Err(e) => {
                                        system.show_error(&format!("Failed to load: {:?}", e));
                                    }
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                system.show_error(&format!("Could not get file name: {:?}", e));
                            }
                        }
                    }
                    if system.save_file_dialog.is_some() && ui.button("Save As...").clicked() {
                        match system.save_file_dialog("pixelpen") {
                            Ok(Some(filename)) => {
                                match storage::save(&doc, std::path::Path::new(&filename)) {
                                    Ok(()) => {}
                                    Err(e) => {
                                        system.show_error(&format!("Failed to save: {:?}", e));
                                    }
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                system.show_error(&format!("Could not get file name: {:?}", e));
                            }
                        }
                    }
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });

            // Top toolbar
            ui.vertical(|ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("Zoom:");
                    if ui.button("-").on_hover_text("Zoom out").clicked() && ui_state.zoom > 1.0 {
                        ui_state.zoom /= 2.0
                    }
                    if ui
                        .button(format!("{:0.0}x", ui_state.zoom))
                        .on_hover_text("Set to 2x")
                        .clicked()
                    {
                        ui_state.zoom = 2.0;
                    }
                    if ui.button("+").on_hover_text("Zoom in").clicked() && ui_state.zoom < 16.0 {
                        ui_state.zoom *= 2.0
                    }
                });
                ui.separator();
                render_palette(ui, &mut doc.paint_color, &mut doc.image);
            });
        });

        // Left toolbar
        egui::SidePanel::left("toolbar", 250.0).show(ctx, |ui| {
            // Main image. ScrollArea unfortunately only provides vertical scrolling.
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    // Import
                    ui.selectable_value(&mut ui_state.mode, Mode::Import, "Import")
                        .on_hover_text("Import image file");
                    ui.shrink_width_to_current();
                    if let Mode::Import = ui_state.mode {
                        render_import(ui, doc, system);
                    }
                    // PixelPaint
                    ui.selectable_value(&mut ui_state.mode, Mode::PixelPaint, "Pixel paint")
                        .on_hover_text("Paint pixels");
                    // ColorPaint
                    ui.selectable_value(&mut ui_state.mode, Mode::ColorPaint, "Color paint")
                        .on_hover_text("Change the color of character cells");
                });
            });
        });

        // Main image.
        egui::CentralPanel::default().show(ctx, |ui| {
            let par = doc.image.pixel_aspect_ratio();
            let (response, painter) = image_painter(ui);
            let pixel_transform = PixelTransform {
                screen_rect: Rect::from_center_size(
                    response.rect.center(),
                    Vec2::new(width as f32 * par * ui_state.zoom, height as f32 * par),
                ),
                pixel_width: width as i32,
                pixel_height: height as i32,
            };

            let hover_pos_screen = ui.input().pointer.tooltip_pos();
            let hover_pos = hover_pos_screen.and_then(|p| pixel_transform.bounded_pixel_pos(p));

            match ui_state.mode {
                Mode::Import => {}
                Mode::PixelPaint | Mode::ColorPaint => {
                    update_in_paint_mode(hover_pos, doc, ui, &response, &pixel_transform, &ui_state)
                }
            }
            // Draw the main image
            let tex_allocator = frame.tex_allocator();

            let texture = update_texture(
                &mut doc.image,
                image_texture,
                tex_allocator,
                par,
                ui_state.zoom,
            );
            let mut mesh = Mesh::with_texture(texture);
            mesh.add_rect_with_uv(
                pixel_transform.screen_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            painter.add(Shape::Mesh(mesh));

            // Highlight character
            if let Mode::ColorPaint = ui_state.mode {
                if let Some(pos) = hover_pos {
                    if let Some((top_left, w, h)) = doc.image.character_box(pos) {
                        painter.rect_stroke(
                            Rect::from_min_max(
                                pixel_transform.screen_pos(top_left),
                                pixel_transform
                                    .screen_pos(Point::new(top_left.x + w, top_left.y + h)),
                            ),
                            0.0,
                            (1.0, vic::palette_color(doc.paint_color)),
                        );
                    }
                }
            }

            let info_text = {
                let t = doc.image.image_info();
                if let Some(p) = hover_pos {
                    format!("{}\n{}", t, doc.image.pixel_info(p))
                } else {
                    t
                }
            };
            painter.text(
                response.rect.left_bottom(),
                Align2::LEFT_BOTTOM,
                &info_text,
                TextStyle::Monospace,
                Color32::WHITE,
            );
        });

        if let Some(doc) = new_doc {
            self.doc = doc;
        }
    }
}

/// Create a Response and Painter for the main image area.
fn image_painter(ui: &mut egui::Ui) -> (Response, Painter) {
    let size = ui.available_size();
    let response = ui.allocate_response(size, egui::Sense::click_and_drag());
    let clip_rect = ui.clip_rect().intersect(response.rect);
    let painter = Painter::new(ui.ctx().clone(), ui.layer_id(), clip_rect);
    (response, painter)
}

fn render_import(ui: &mut egui::Ui, doc: &mut Document, system: &mut SystemFunctions) {
    ui.vertical(|ui| {
        ui.horizontal_for_text(TextStyle::Body, |ui| {
            let mut filename = doc.import.filename.clone().unwrap_or_default();
            ui.label("File name:");
            ui.vertical(|ui| {
                ui.text_edit_singleline(&mut filename);
                let filename = if ui.button("Browse...").clicked() {
                    if let Ok(Some(f)) = system.open_file_dialog() {
                        f.to_str().map(str::to_string)
                    } else {
                        None
                    }
                } else {
                    Some(filename.trim().to_string())
                };
                doc.import.filename = match filename {
                    Some(f) if f.is_empty() => None,
                    Some(f) => Some(f),
                    None => None,
                };
            });
        });
    });
}

fn update_in_paint_mode(
    hover_pos: Option<Point>,
    doc: &mut Document,
    ui: &mut egui::Ui,
    response: &egui::Response,
    pixel_transform: &PixelTransform,
    ui_state: &UiState,
) {
    let disallowed_message =
        &hover_pos.and_then(|hover_pos| doc.image.check_allowed_paint(doc.paint_color, hover_pos));
    if let Some(message) = disallowed_message {
        egui::popup::show_tooltip_text(ui.ctx(), egui::Id::new("not_allowed"), message);
    }
    if let Some(pointer_pos) = response.interact_pointer_pos() {
        if disallowed_message.is_none() {
            let pointer = &response.ctx.input().pointer;
            let color_to_set = if pointer.button_down(egui::PointerButton::Secondary) {
                doc.image.colors[GlobalColors::BACKGROUND]
            } else {
                doc.paint_color as u8
            };
            if let Some(Point { x, y }) = pixel_transform.bounded_pixel_pos(pointer_pos) {
                match ui_state.mode {
                    Mode::PixelPaint => {
                        doc.image.set_pixel(x, y, color_to_set);
                    }
                    Mode::ColorPaint => {
                        doc.image.set_color(x, y, color_to_set);
                    }
                    _ => panic!(
                        "update_in paint_mode with invalid mode: {:?}",
                        ui_state.mode
                    ),
                }
            }
        }
    }
}

fn render_palette(
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
            ui::palette::palette_patch(
                ui.painter(),
                &patch_rect,
                color,
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

/// Updates the texture with the current image content, if needed.
/// Returns the texture id.
fn update_texture(
    image: &mut MutationMonitor<VicImage>,
    image_texture: &mut Option<Texture>,
    tex_allocator: &mut dyn TextureAllocator,
    par: f32,
    zoom: f32,
) -> TextureId {
    let scale_x = ((par * zoom).ceil() as u32).max(1).min(MAX_SCALE);
    let scale_y = (zoom.ceil() as u32).max(1).min(MAX_SCALE);
    let (source_width, source_height) = image.pixel_size();
    let texture_width = source_width * scale_x as usize;
    let texture_height = source_height * scale_y as usize;

    // Recreate the texture if the size has changed or the image has been updated
    if let Some(t) = image_texture {
        if t.width != texture_width || t.height != texture_height || image.dirty {
            tex_allocator.free(t.id);
            *image_texture = None;
        }
    }
    if image.dirty {
        image.update();
    }
    let texture = if let Some(texture) = image_texture {
        texture.id
    } else {
        let mut unscaled_pixels = ImgVec::new(
            vec![Color32::BLACK; source_width * source_height],
            source_width,
            source_height,
        );
        image.render(unscaled_pixels.as_mut());
        let mut pixels = scaling::scale_image(unscaled_pixels.as_ref(), scale_x, scale_y);

        let texture_id = tex_allocator.alloc_srgba_premultiplied(
            (texture_width, texture_height),
            &pixels.as_contiguous_buf().0,
        );
        *image_texture = Some(Texture {
            id: texture_id,
            width: texture_width,
            height: texture_height,
        });
        texture_id
    };
    image.dirty = false;
    texture
}

impl Application {
    pub fn with_doc(doc: Document) -> Self {
        Application {
            ui_state: UiState {
                mode: Mode::PixelPaint,
                zoom: 2.0,
            },
            doc,
            image_texture: None,
            system: Default::default(),
        }
    }
}
