//! User interface for the Import tool.

use std::path::Path;

use crate::actions::Action;
use crate::actions::DocAction;
use crate::actions::UiAction;
use crate::cell_image::CellImageSize;
use crate::coords::PixelPoint;
use crate::coords::PixelTransform;
use crate::import::Import;
use crate::import::ImportSettings;
use crate::import::PixelAspectRatio;
use crate::tool::Tool;
use crate::vic::ColorFormat;
use crate::Document;
use eframe::egui;
use eframe::egui::Color32;
use eframe::egui::ComboBox;
use eframe::egui::DragValue;
use eframe::egui::Label;
use eframe::egui::Painter;
use eframe::egui::Stroke;
use image::imageops::FilterType;
use image::GenericImageView;

const IMPORT_IMAGE_EXTENTS_COLOR: Color32 = Color32::GRAY;
const UNKNOWN_SOURCE_TEXT: &str = "unknown source";

#[derive(Clone)]
pub struct ImportTool {
    import: Import,
}

impl ImportTool {
    pub fn new(import: Import) -> Self {
        Self { import }
    }
    pub fn filename(&self) -> Option<&Path> {
        self.import.settings.filename.as_deref()
    }
    pub fn update_ui(
        &mut self,
        ctx: &egui::CtxRef,
        _ui: &mut egui::Ui,
        painter: &Painter,
        doc: &Document,
        pixel_transform: &PixelTransform,
        user_actions: &mut Vec<Action>,
    ) {
        image_ui(painter, &mut self.import, pixel_transform);
        egui::Window::new("Import")
            .show(ctx, |ui| tool_ui(ui, doc, &mut self.import, user_actions));
    }
}

fn image_ui(painter: &Painter, import: &mut Import, transform: &PixelTransform) {
    let ImportSettings {
        left,
        top,
        width,
        height,
        ..
    } = import.settings;
    let rect = egui::Rect::from_min_max(
        transform.screen_pos(PixelPoint::new(left, top)),
        transform.screen_pos(PixelPoint::new(left + width as i32, top + height as i32)),
    );
    let stroke = Stroke::new(1.0, IMPORT_IMAGE_EXTENTS_COLOR);
    painter.rect_stroke(rect, 0.0, stroke);
    painter.line_segment([rect.left_top(), rect.right_bottom()], stroke);
    painter.line_segment([rect.left_bottom(), rect.right_top()], stroke);
}

/// Render the tool UI.
fn tool_ui(ui: &mut egui::Ui, doc: &Document, import: &mut Import, user_actions: &mut Vec<Action>) {
    egui::Grid::new("import_grid").show(ui, |ui| {
        let source = &import.image;
        let target = &doc.image;
        let (source_width, source_height) = source.dimensions();
        let (target_width, target_height) = target.size_in_pixels();

        ui.label("Source");
        ui.label(format!(
            "{}\n({}x{} pixels)",
            import
                .settings
                .filename
                .clone()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| UNKNOWN_SOURCE_TEXT.to_string()),
            source_width,
            source_height
        ));
        ui.end_row();

        ui.add(Label::new("Pixel size"))
            .on_hover_text("Aspect ratio of pixels in imported image");
        ComboBox::from_id_source("import_pixel_size")
            .selected_text(format!("{}", import.settings.pixel_aspect_ratio))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut import.settings.pixel_aspect_ratio,
                    PixelAspectRatio::Square,
                    format!("{}", PixelAspectRatio::Square),
                );
                ui.selectable_value(
                    &mut import.settings.pixel_aspect_ratio,
                    PixelAspectRatio::Target,
                    format!("{}", PixelAspectRatio::Target),
                );
                ui.selectable_value(
                    &mut import.settings.pixel_aspect_ratio,
                    PixelAspectRatio::TargetHalfResolution,
                    format!("{}", PixelAspectRatio::TargetHalfResolution),
                );
            });
        ui.end_row();

        ui.add(Label::new("Left"));
        ui.add(
            DragValue::new(&mut import.settings.left)
                .clamp_range(-(import.settings.width as f32)..=target_width as f32 - 1.0),
        );
        ui.end_row();

        ui.add(Label::new("Top"));
        ui.add(
            DragValue::new(&mut import.settings.top)
                .clamp_range(-(import.settings.height as f32)..=target_height as f32 - 1.0),
        );
        ui.end_row();

        ui.add(Label::new("Width"));
        ui.add(
            DragValue::new(&mut import.settings.width).clamp_range(1.0..=target_width as f32 * 4.0),
        );
        ui.end_row();

        import.settings.height = (match import.settings.pixel_aspect_ratio {
            PixelAspectRatio::Square => {
                import.settings.width as f32 / source_width as f32
                    * source_height as f32
                    * target.pixel_aspect_ratio()
            }
            PixelAspectRatio::Target => {
                import.settings.width as f32 / source_width as f32 * source_height as f32
            }
            PixelAspectRatio::TargetHalfResolution => {
                import.settings.width as f32 / source_width as f32 * source_height as f32 / 2.0
            }
        }
        .round() as u32)
            .max(1);

        ui.label("Height");
        ui.label(format!("{}", import.settings.height));
        ui.end_row();

        ui.label("Scaling filter");
        ComboBox::from_id_source("import_scaling_filter")
            .selected_text(format!("{:?}", import.settings.filter))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut import.settings.filter, FilterType::Nearest, "Nearest");
                ui.selectable_value(
                    &mut import.settings.filter,
                    FilterType::Triangle,
                    "Triangle",
                );
                ui.selectable_value(
                    &mut import.settings.filter,
                    FilterType::CatmullRom,
                    "CatmullRom",
                );
                ui.selectable_value(
                    &mut import.settings.filter,
                    FilterType::Gaussian,
                    "Gaussian",
                );
                ui.selectable_value(
                    &mut import.settings.filter,
                    FilterType::Lanczos3,
                    "Lanczos3",
                );
            });
        ui.end_row();

        ui.label("Format");
        ComboBox::from_id_source("import_color_format")
            .selected_text(match import.settings.format {
                ColorFormat::HighRes => "High Resolution",
                ColorFormat::Multicolor => "Multicolor",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut import.settings.format,
                    ColorFormat::Multicolor,
                    "Multicolor",
                );
                ui.selectable_value(
                    &mut import.settings.format,
                    ColorFormat::HighRes,
                    "High Resolution",
                );
            });
        ui.end_row();
    });
    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Import").clicked() {
            let scaled = import.scale_image();
            user_actions.push(Action::Document(DocAction::PasteTrueColor {
                source: scaled,
                target: PixelPoint::new(import.settings.left, import.settings.top),
                format: import.settings.format,
            }));
        } else if ui.button("Close").clicked() {
            user_actions.push(Action::Ui(UiAction::SelectTool(Tool::Paint(
                Default::default(),
            ))));
        }
    });
    ui.end_row();
}
