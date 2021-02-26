use eframe::{
    egui::{self, paint::Mesh, Button, Color32, Pos2, Rect, Sense, Shape, Stroke, TextureId, Vec2},
    epi,
};

#[derive(PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
enum Mode {
    PixelPaint,
    CharPaint,
}

// From /usr/lib/vice/VIC20/vice.vpl
const PALETTE: [u32; 16] = [
    // 0xRRGGBB
    0x000000, // Black
    0xffffff, // White
    0xf00000, // Red
    0x00f0f0, // Cyan
    0x600060, // Purple
    0x00a000, // Green
    0x0000f0, // Blue
    0xd0d000, // Yellow
    0xc0a000, // Orange
    0xffa000, // Light Orange
    0xf08080, // Pink
    0x00ffff, // Light Cyan
    0xff00ff, // Light Purple
    0x00ff00, // Light Green
    0x00a0ff, // Light Blue
    0xffff00, // Light Yellow
];

fn selected_color(selected: bool) -> Option<Color32> {
    if selected {
        Some(Color32::BLUE)
    } else {
        None
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct TemplateApp {
    mode: Mode,
    paint_color: usize,

    width: usize,
    height: usize,
    pixels: Vec<Color32>,

    #[cfg_attr(feature = "persistence", serde(skip))]
    image_texture: Option<TextureId>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let width = 320;
        let height = 200;
        let mut pixels = vec![Color32::RED; width * height];
        for x in 0..width {
            let y = ((x as f32 / width as f32) * height as f32) as usize;
            pixels[x + y * width] = Color32::WHITE;
        }
        Self {
            mode: Mode::PixelPaint,
            paint_color: 0,
            width,
            height,
            pixels,
            image_texture: None,
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Paint Application"
    }

    /// Called by the framework to load old app state (if any).
    #[cfg(feature = "persistence")]
    fn load(&mut self, storage: &dyn epi::Storage) {
        *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let TemplateApp {
            mode,
            paint_color,
            width,
            height,
            image_texture,
            pixels,
            ..
        } = self;

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
                    for (color_index, rgb) in PALETTE.iter().enumerate() {
                        let color = Color32::from_rgb(
                            (rgb >> 16) as u8,
                            ((rgb >> 8) & 0xff) as u8,
                            (rgb & 0xff) as u8,
                        );
                        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
                        ui.painter().rect_filled(rect, 0.0, color);
                        if color_index == *paint_color {
                            ui.painter()
                                .rect_stroke(rect, 0.0, Stroke::new(2.0, Color32::BLUE));
                        }
                        if response.clicked() {
                            *paint_color = color_index;
                        }
                    }
                }
                Mode::CharPaint => {}
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Main image. ScrollArea unfortunately only provides vertical scrolling.
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                let zoom = 2.0;
                let size = Vec2::new(*width as f32, *height as f32) * zoom;

                let (response, painter) = ui.allocate_painter(size, egui::Sense::drag());

                let tex_allocator = frame.tex_allocator().as_mut().unwrap();

                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let p = pointer_pos - response.rect.left_top();
                    let fx = p.x / response.rect.size().x;
                    let fy = p.y / response.rect.size().y;
                    let x = (fx * *width as f32).round() as i32;
                    let y = (fy * *height as f32).round() as i32;
                    if x >= 0 && (x as usize) < *width && y >= 0 && (y as usize) < *height {
                        pixels[x as usize + y as usize * *width] = Color32::LIGHT_GRAY;
                        tex_allocator.free(image_texture.take().unwrap()); // make sure we create a new texture
                    }
                }

                let texture = if let Some(texture) = image_texture {
                    *texture
                } else {
                    let texture =
                        tex_allocator.alloc_srgba_premultiplied((*width, *height), &pixels);
                    *image_texture = Some(texture);
                    texture
                };

                let mut mesh = Mesh::with_texture(texture);
                mesh.add_rect_with_uv(
                    response.rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                painter.add(Shape::Mesh(mesh));
            });
        });
    }
}
