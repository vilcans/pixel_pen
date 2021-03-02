use eframe::{
    egui::{self, paint::Mesh, Button, Color32, Pos2, Rect, Sense, Shape, Stroke, TextureId, Vec2},
    epi,
};

use crate::{
    mutation_monitor::MutationMonitor,
    vic::{self, GlobalColors, VicImage},
    widgets,
};

#[derive(PartialEq)]
enum Mode {
    PixelPaint,
    CharPaint,
}

fn selected_color(selected: bool) -> Option<Color32> {
    if selected {
        Some(Color32::BLUE)
    } else {
        None
    }
}

pub struct Application {
    mode: Mode,
    paint_color: usize,

    image: MutationMonitor<VicImage>,

    image_texture: Option<TextureId>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            mode: Mode::PixelPaint,
            paint_color: 1,
            image: MutationMonitor::new_dirty(VicImage::default()),
            image_texture: None,
        }
    }
}

impl epi::App for Application {
    fn name(&self) -> &str {
        "Paint Application"
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Application {
            mode,
            paint_color,
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
                if ui
                    .add(Button::new("Pixel paint").fill(selected_color(*mode == Mode::PixelPaint)))
                    .clicked()
                {
                    *mode = Mode::PixelPaint;
                }
                if ui
                    .add(Button::new("Char paint").fill(selected_color(*mode == Mode::CharPaint)))
                    .clicked()
                {
                    *mode = Mode::CharPaint;
                }
            });

            // Color selector
            ui.horizontal_wrapped(|ui| match *mode {
                Mode::PixelPaint => {
                    let size = ui.spacing().interact_size;
                    for color_index in 0..vic::PALETTE_SIZE {
                        let color = vic::palette_color(color_index);
                        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                        ui.painter().rect_filled(rect, 0.0, color);
                        if color_index == *paint_color {
                            ui.painter()
                                .rect_stroke(rect, 0.0, Stroke::new(2.0, Color32::BLUE));
                        }
                        if response.clicked() {
                            *paint_color = color_index;
                        }
                        let popup_id =
                            ui.make_persistent_id(format!("color_popup_{}", color_index));
                        if response.secondary_clicked() {
                            ui.memory().open_popup(popup_id);
                        }
                        widgets::popup(ui, popup_id, &response, |ui| {
                            let color_index = color_index as u8;
                            ui.label(format!("Color {0} (${0:x})", color_index));
                            for (index, label, range) in vic::GLOBAL_COLORS.iter() {
                                let index = *index as u32;
                                if range.contains(&color_index) {
                                    let setting = image.colors[index];
                                    let mut selected = setting == color_index;
                                    if ui.checkbox(&mut selected, *label).clicked()
                                        && setting != color_index
                                    {
                                        println!("Setting {0} to {1}", label, color_index);
                                        image.colors[index] = color_index;
                                    }
                                }
                            }
                        });
                    }
                }
                Mode::CharPaint => {}
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Main image. ScrollArea unfortunately only provides vertical scrolling.
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                let zoom = 2.0;
                let par = image.pixel_aspect_ratio();
                let size = Vec2::new(width as f32 * par, height as f32) * zoom;

                let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());

                let tex_allocator = frame.tex_allocator();

                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let pointer = &response.ctx.input().pointer;
                    let color_to_set = if pointer.button_down(egui::PointerButton::Secondary) {
                        image.colors[GlobalColors::BACKGROUND]
                    } else {
                        *paint_color as u8
                    };
                    let p = pointer_pos - response.rect.left_top();
                    let fx = p.x / response.rect.size().x;
                    let fy = p.y / response.rect.size().y;
                    let x = (fx * width as f32).round() as i32;
                    let y = (fy * height as f32).round() as i32;
                    if x >= 0 && (x as usize) < width && y >= 0 && (y as usize) < height {
                        image.set_pixel(x, y, color_to_set);
                    }
                }

                // Draw the main image
                if image.dirty {
                    if let Some(t) = image_texture.take() {
                        tex_allocator.free(t);
                    }
                    image.update();
                }
                let texture = if let Some(texture) = image_texture {
                    *texture
                } else {
                    image.render();
                    let pixels = image.pixels();
                    let texture = tex_allocator.alloc_srgba_premultiplied((width, height), &pixels);
                    *image_texture = Some(texture);
                    texture
                };
                image.dirty = false;

                let mut mesh = Mesh::with_texture(texture);
                mesh.add_rect_with_uv(
                    response.rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.add(Shape::Mesh(mesh));

                ui.label(image.info());
            });
        });
    }
}
