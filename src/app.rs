use std::time::Instant;

use crate::{
    coords::{PixelTransform, Point},
    document::Document,
    import::{self, Import},
    mutation_monitor::MutationMonitor,
    scaling, storage,
    system::SystemFunctions,
    ui,
    vic::{self, GlobalColors, VicImage},
    widgets,
};
use eframe::{
    egui::{
        self, paint::Mesh, Align2, Color32, CursorIcon, Label, Painter, PointerButton, Pos2, Rect,
        Response, Rgba, Sense, Shape, Stroke, TextStyle, TextureId, Vec2,
    },
    epi::{self, TextureAllocator},
};
use imgref::ImgVec;
use itertools::Itertools;

// Don't scale the texture more than this to avoid huge textures when zooming.
const MAX_SCALE: u32 = 8;

const POPUP_MESSAGE_TIME: f32 = 3.0;
const POPUP_HIGHLIGHT_TIME: f32 = 0.3;

const BORDER_CORNER_RADIUS: f32 = 15.0;
const BORDER_SIZE: Vec2 = Vec2::new(25.0, 20.0);

const GRID_COLOR: Color32 = Color32::GRAY;

#[derive(Debug)]
enum Mode {
    Import(Import),
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
    /// Enable showing the character grid
    grid: bool,
    /// Whether user is currently panning
    panning: bool,
    pan: Vec2,

    message: Option<(Instant, String)>,
}
impl UiState {
    fn show_warning(&mut self, message: String) {
        self.message = Some((Instant::now(), message));
    }
}

#[derive(Default)]
struct UserActions {
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub toggle_grid: bool,
}
impl UserActions {
    pub fn update_from_text(&mut self, t: &str) {
        match t {
            "+" => self.zoom_in = true,
            "-" => self.zoom_out = true,
            "g" => self.toggle_grid = true,
            _ => (),
        }
    }
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
            doc,
            ui_state,
            image_texture,
            system,
            ..
        } = self;

        let (width, height) = doc.image.pixel_size();
        let mut new_doc = None;
        let mut cursor_icon = None;

        let mut user_actions = UserActions::default();
        for e in ctx.input().events.iter() {
            if !ctx.wants_keyboard_input() {
                if let egui::Event::Text(t) = e {
                    user_actions.update_from_text(&t)
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
                    if system.open_file_dialog.is_some() && ui.button("Import...").clicked() {
                        match system.open_file_dialog() {
                            Ok(Some(filename)) => {
                                // TODO: get rid of unwrap, use PathBuf instead of String for file names
                                let filename = filename.into_os_string().into_string().unwrap();
                                match Import::load(&filename) {
                                    Ok(i) => ui_state.mode = Mode::Import(i),
                                    Err(e) => system.show_error(&format!(
                                        "Could not import file {}: {:?}",
                                        filename, e
                                    )),
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                system.show_error(&format!("Could not get file name: {:?}", e))
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
                    user_actions.zoom_out |= ui.button("-").on_hover_text("Zoom out").clicked();
                    if ui
                        .button(format!("{:0.0}x", ui_state.zoom))
                        .on_hover_text("Set to 2x")
                        .clicked()
                    {
                        ui_state.zoom = 2.0;
                    }
                    user_actions.zoom_in |= ui.button("+").on_hover_text("Zoom in").clicked();
                    ui.separator();
                    ui.checkbox(&mut ui_state.grid, "Grid");
                });
                ui.separator();
                render_palette(ui, &mut doc.paint_color, &mut doc.image);
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if let Some((time, message)) = ui_state.message.as_ref() {
                let age = Instant::now()
                    .saturating_duration_since(*time)
                    .as_secs_f32();
                let highlight = 1.0 - (age / POPUP_HIGHLIGHT_TIME).clamp(0.0, 1.0);
                let color = Rgba::RED * highlight;
                ui.add(Label::new(message).background_color(color));
                if age >= POPUP_MESSAGE_TIME {
                    ui_state.message = None;
                } else if highlight > 0.0 {
                    ctx.request_repaint(); // to animate color highlight
                }
            } else {
                ui.label("Have fun!");
            }
        });

        // Left toolbar
        egui::SidePanel::left("toolbar")
            .default_width(250.0)
            .show(ctx, |ui| {
                egui::ScrollArea::auto_sized().show(ui, |ui| {
                    ui.vertical_centered_justified(|ui| {
                        // PixelPaint
                        if ui
                            .selectable_label(
                                matches!(ui_state.mode, Mode::PixelPaint),
                                "Pixel paint",
                            )
                            .on_hover_text("Paint pixels")
                            .clicked()
                        {
                            ui_state.mode = Mode::PixelPaint;
                        }
                        // ColorPaint
                        if ui
                            .selectable_label(
                                matches!(ui_state.mode, Mode::ColorPaint),
                                "Color paint",
                            )
                            .on_hover_text("Change the color of character cells")
                            .clicked()
                        {
                            ui_state.mode = Mode::ColorPaint;
                        }
                    });
                });
            });

        // Main image.
        egui::CentralPanel::default().show(ctx, |ui| {
            let par = doc.image.pixel_aspect_ratio();
            let (response, painter) = image_painter(ui);
            let pixel_transform = PixelTransform {
                screen_rect: Rect::from_center_size(
                    response.rect.center() + ui_state.pan,
                    Vec2::new(
                        width as f32 * par * ui_state.zoom,
                        height as f32 * ui_state.zoom,
                    ),
                ),
                pixel_width: width as i32,
                pixel_height: height as i32,
            };

            let hover_pos_screen = ui.input().pointer.hover_pos();
            let hover_pos = hover_pos_screen.and_then(|p| pixel_transform.bounded_pixel_pos(p));

            let input = ui.input();
            if input.modifiers.command {
                if input.scroll_delta.y < 0.0 {
                    user_actions.zoom_out = true;
                } else if input.scroll_delta.y > 0.0 {
                    user_actions.zoom_in = true;
                }
            } else {
                ui_state.pan += input.scroll_delta;
            }

            if response.drag_started() && input.pointer.button_down(PointerButton::Middle)
                || (input.pointer.button_down(PointerButton::Secondary) && input.modifiers.shift)
            {
                ui_state.panning = true;
            }
            if ui_state.panning {
                ui_state.pan += input.pointer.delta();
                cursor_icon = Some(CursorIcon::Grabbing);
            } else {
                match ui_state.mode {
                    Mode::Import(_) => {}
                    Mode::PixelPaint | Mode::ColorPaint => {
                        update_in_paint_mode(
                            hover_pos,
                            doc,
                            ui,
                            &response,
                            &pixel_transform,
                            ui_state,
                            &mut cursor_icon,
                        );
                    }
                }
            }
            if response.drag_released() {
                ui_state.panning = false;
            }

            painter.rect_filled(
                pixel_transform
                    .screen_rect
                    .expand2(BORDER_SIZE * ui_state.zoom),
                BORDER_CORNER_RADIUS * ui_state.zoom,
                doc.image.border(),
            );

            // Draw the main image
            let texture = update_texture(
                &mut doc.image,
                image_texture,
                frame.tex_allocator(),
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

            // Grid lines
            if ui_state.grid {
                draw_grid(&doc.image, &painter, &pixel_transform);
            }

            // Import preview
            if let Mode::Import(import) = &mut ui_state.mode {
                import::image_ui(ui, &painter, import, &pixel_transform);
            }

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
                    format!("{}\n{}", doc.image.pixel_info(p), t)
                } else {
                    t
                }
            };
            painter.text(
                response.rect.left_bottom(),
                Align2::LEFT_BOTTOM,
                &info_text,
                TextStyle::Monospace,
                Color32::from_rgb(0x88, 0x88, 0x88),
            );
        });

        if let Mode::Import(import) = &mut ui_state.mode {
            let mut keep_open = true;
            egui::Window::new("Import").show(ctx, |ui| {
                keep_open = import::tool_ui(ui, doc, import);
            });
            if !keep_open {
                ui_state.mode = Mode::PixelPaint;
            }
        }

        if user_actions.zoom_in && ui_state.zoom < 16.0 {
            ui_state.zoom *= 2.0
        }
        if user_actions.zoom_out && ui_state.zoom > 1.0 {
            ui_state.zoom /= 2.0
        }
        if user_actions.toggle_grid {
            ui_state.grid = !ui_state.grid;
        }

        if let Some(doc) = new_doc {
            self.doc = doc;
        }

        if let Some(icon) = cursor_icon {
            ctx.output().cursor_icon = icon;
        }
    }
}

fn draw_grid(image: &VicImage, painter: &Painter, pixel_transform: &PixelTransform) {
    let (width, height) = image.pixel_size();
    let stroke = Stroke {
        width: 1.0,
        color: GRID_COLOR,
    };
    for x in image.vertical_grid_lines() {
        painter.line_segment(
            [
                pixel_transform.screen_pos(Point { x, y: 0 }),
                pixel_transform.screen_pos(Point {
                    x,
                    y: height as i32,
                }),
            ],
            stroke,
        )
    }
    for y in image.horizontal_grid_lines() {
        painter.line_segment(
            [
                pixel_transform.screen_pos(Point { x: 0, y }),
                pixel_transform.screen_pos(Point { x: width as i32, y }),
            ],
            stroke,
        )
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

fn update_in_paint_mode(
    pixel_pos: Option<Point>,
    doc: &mut Document,
    ui: &mut egui::Ui,
    response: &egui::Response,
    _pixel_transform: &PixelTransform,
    ui_state: &mut UiState,
    cursor_icon: &mut Option<CursorIcon>,
) {
    if pixel_pos.is_none() {
        return;
    }
    let hover_pos = pixel_pos.unwrap();

    *cursor_icon = Some(CursorIcon::PointingHand);

    let color = if response.secondary_clicked()
        || (response.dragged() && ui.input().pointer.button_down(PointerButton::Secondary))
    {
        Some(doc.image.colors[GlobalColors::BACKGROUND] as usize)
    } else if response.clicked() || response.dragged() {
        Some(doc.paint_color)
    } else {
        None
    };
    if let Some(color) = color {
        let disallowed_message = doc.image.check_allowed_paint(color, hover_pos);
        if let Some(message) = disallowed_message {
            ui_state.show_warning(message);
        } else {
            let Point { x, y } = hover_pos;
            let was_dirty = doc.image.dirty;
            let changed = match ui_state.mode {
                Mode::PixelPaint => doc.image.set_pixel(x, y, color as u8),
                Mode::ColorPaint => doc.image.set_color(x, y, color as u8),
                _ => panic!(
                    "update_in paint_mode with invalid mode: {:?}",
                    ui_state.mode
                ),
            };
            if !changed {
                doc.image.dirty = was_dirty;
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
                grid: false,
                panning: false,
                pan: Vec2::ZERO,
                message: None,
            },
            doc,
            image_texture: None,
            system: Default::default(),
        }
    }
}
