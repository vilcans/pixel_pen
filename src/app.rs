use eframe::{
    egui::{self, paint::Mesh, Button, Color32, Rect, Sense, Shape, Stroke, TextureId},
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
    // Example stuff:
    label: String,
    value: f32,
    painting: Painting,
    mode: Mode,
    paint_color: usize,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            painting: Default::default(),
            mode: Mode::PixelPaint,
            paint_color: 0,
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "egui template"
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
            label,
            value,
            painting,
            mode,
            paint_color,
        } = self;

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::SidePanel::left("side_panel", 200.0).show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::f32(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(
                    egui::Hyperlink::new("https://github.com/emilk/egui/").text("powered by egui"),
                );
            });
        });

        egui::TopPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
            ui.separator();
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
            ui.heading("egui template");
            ui.hyperlink("https://github.com/emilk/egui_template");
            ui.add(egui::github_link_file_line!(
                "https://github.com/emilk/egui_template/blob/master/",
                "Direct link to source code."
            ));
            egui::warn_if_debug_build(ui);

            ui.separator();

            ui.heading("Central Panel");
            ui.label("The central panel the region left after adding TopPanel's and SidePanel's");
            ui.label("It is often a great place for big things, like drawings:");

            ui.heading("Draw with your mouse to paint:");
            painting.ui_control(ui);
            egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                painting.ui_content(ui);
            });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

// ----------------------------------------------------------------------------

/// Example code for painting on a canvas with your mouse
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
struct Painting {
    lines: Vec<Vec<egui::Vec2>>,
    stroke: egui::Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
        }
    }
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            egui::stroke_ui(ui, &mut self.stroke, "Stroke");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            [100.0, 100.0].into(), /*ui.available_size_before_wrap_finite()*/
            egui::Sense::drag(),
        );
        let rect = response.rect;

        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = pointer_pos - rect.min;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
            }
        } else if !current_line.is_empty() {
            self.lines.push(vec![]);
        }

        let mut shapes = vec![];
        for line in &self.lines {
            if line.len() >= 2 {
                let points: Vec<egui::Pos2> = line.iter().map(|p| rect.min + *p).collect();
                for p in &points {
                    let mut m = Mesh::with_texture(TextureId::Egui);
                    m.add_colored_rect(
                        Rect::from_center_size(*p, [10.0, 10.0].into()),
                        Color32::RED,
                    );
                    shapes.push(Shape::Mesh(m));
                }
                shapes.push(egui::Shape::line(points, self.stroke));
            }
        }
        painter.extend(shapes);

        response
    }
}
