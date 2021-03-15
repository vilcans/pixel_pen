use std::path::Path;

use eframe::{
    egui::{self, paint::Mesh, Color32, Pos2, Rect, Sense, Shape, TextureId, Vec2},
    epi::{self, TextureAllocator},
};
use itertools::Itertools;

use crate::{
    coords::{PixelTransform, Point},
    error::Error,
    image_io,
    mutation_monitor::MutationMonitor,
    scaling, ui,
    vic::{self, GlobalColors, VicImage},
    widgets,
};

// Don't scale the texture more than this to avoid huge textures when zooming.
const MAX_SCALE: u32 = 8;

#[derive(PartialEq)]
enum Mode {
    PixelPaint,
    ColorPaint,
}

struct Texture {
    pub id: TextureId,
    pub width: usize,
    pub height: usize,
}

pub struct Application {
    mode: Mode,
    paint_color: usize,
    zoom: f32,

    image: MutationMonitor<VicImage>,
    image_texture: Option<Texture>,
}

impl Default for Application {
    fn default() -> Self {
        Self::with_image(VicImage::default())
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
            mode,
            paint_color,
            zoom,
            image,
            image_texture,
            ..
        } = self;
        let (width, height) = image.pixel_size();

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // Menu bar
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });

            // Toolbar
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(mode, Mode::PixelPaint, "Pixel paint")
                    .on_hover_text("Paint pixels");
                ui.selectable_value(mode, Mode::ColorPaint, "Color paint")
                    .on_hover_text("Change the color of character cells");
                ui.separator();
                ui.label("Zoom:");
                if ui.button("-").on_hover_text("Zoom out").clicked() && *zoom > 1.0 {
                    *zoom /= 2.0
                }
                if ui
                    .button(format!("{:0.0}x", *zoom))
                    .on_hover_text("Set to 2x")
                    .clicked()
                {
                    *zoom = 2.0;
                }
                if ui.button("+").on_hover_text("Zoom in").clicked() && *zoom < 16.0 {
                    *zoom *= 2.0
                }
            });

            ui.separator();
            render_palette(ui, paint_color, image);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Main image. ScrollArea unfortunately only provides vertical scrolling.
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                let par = image.pixel_aspect_ratio();
                let size = Vec2::new(width as f32 * par, height as f32) * *zoom;

                let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());

                let pixel_transform = PixelTransform {
                    screen_rect: response.rect,
                    pixel_width: width as i32,
                    pixel_height: height as i32,
                };

                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let pointer = &response.ctx.input().pointer;
                    let color_to_set = if pointer.button_down(egui::PointerButton::Secondary) {
                        image.colors[GlobalColors::BACKGROUND]
                    } else {
                        *paint_color as u8
                    };
                    if let Some(Point { x, y }) = pixel_transform.bounded_pixel_pos(pointer_pos) {
                        match mode {
                            Mode::PixelPaint => {
                                image.set_pixel(x, y, color_to_set);
                            }
                            Mode::ColorPaint => {
                                image.set_color(x, y, color_to_set);
                            }
                        }
                    }
                }

                // Draw the main image
                let tex_allocator = frame.tex_allocator();

                let texture = update_texture(image, image_texture, tex_allocator, par, *zoom);
                let mut mesh = Mesh::with_texture(texture);
                mesh.add_rect_with_uv(
                    response.rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.add(Shape::Mesh(mesh));

                // Highlight character
                match mode {
                    Mode::ColorPaint => {
                        if let Some(pos) = ui.input().pointer.tooltip_pos() {
                            let pixel_pos = pixel_transform.pixel_pos(pos);
                            if let Some((top_left, w, h)) = image.character_box(pixel_pos) {
                                painter.rect_stroke(
                                    Rect::from_min_max(
                                        pixel_transform.screen_pos(top_left),
                                        pixel_transform
                                            .screen_pos(Point::new(top_left.x + w, top_left.y + h)),
                                    ),
                                    0.0,
                                    (1.0, vic::palette_color(*paint_color)),
                                );
                            }
                        }
                    }
                    _ => {}
                }

                ui.label(image.info());
            });
        });
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
                if usages.len() != 0 {
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
        image.render();

        let mut pixels = scaling::scale_image(image.pixels(), scale_x, scale_y);

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
    pub fn with_image(image: VicImage) -> Self {
        Application {
            mode: Mode::PixelPaint,
            paint_color: 1,
            zoom: 2.0,
            image: MutationMonitor::new_dirty(image),
            image_texture: None,
        }
    }

    pub fn load(filename: &Path) -> Result<Application, Error> {
        let image = image_io::load_file(filename)?;
        Ok(Self::with_image(image))
    }
}
