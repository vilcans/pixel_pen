use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use crate::actions::Action;
use crate::actions::DocAction;
use crate::actions::UiAction;
use crate::coords::PixelTransform;
use crate::coords::Point;
use crate::error::Error;
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
use image::DynamicImage;
use image::GenericImageView;
use image::RgbaImage;
use serde::{Deserialize, Serialize};

const IMPORT_IMAGE_EXTENTS_COLOR: Color32 = Color32::GRAY;
const UNKNOWN_SOURCE_TEXT: &str = "unknown source";

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "FilterType")]
enum FilterTypeForSerialization {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum PixelAspectRatio {
    Square,
    Target,
    TargetHalfResolution,
}

impl Display for PixelAspectRatio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PixelAspectRatio::Square => "Square",
                PixelAspectRatio::Target => "Target",
                PixelAspectRatio::TargetHalfResolution => "Target low-res",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ImportSettings {
    #[serde(default)]
    pub filename: Option<PathBuf>,

    #[serde(with = "FilterTypeForSerialization")]
    pub filter: FilterType,

    pub format: ColorFormat,

    /// Aspect ratio to assume for the source pixels
    pub pixel_aspect_ratio: PixelAspectRatio,

    // Placement in target image's pixel coordinates
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
}

/// State of an ongoing import.
#[derive(Debug)]
pub struct Import {
    pub settings: ImportSettings,
    pub image: DynamicImage,
}

impl Import {
    pub fn load(filename: &Path) -> Result<Import, Error> {
        let image = match image::open(filename) {
            Ok(image) => image,
            Err(e) => {
                return Err(Error::ImageError(e));
            }
        };
        println!(
            "Import image {}: dimensions {:?}, colors {:?}",
            filename.display(),
            image.dimensions(),
            image.color()
        );

        Ok(Import {
            settings: ImportSettings {
                filename: Some(filename.to_owned()),
                filter: FilterType::Gaussian,
                format: ColorFormat::Multicolor,
                pixel_aspect_ratio: PixelAspectRatio::Square,
                left: 0,
                top: 0,
                width: image.dimensions().0,
                height: image.dimensions().1,
            },
            image,
        })
    }

    /// Get the scaled image
    pub fn scale_image(&self) -> RgbaImage {
        let settings = &self.settings;
        image::imageops::resize(
            &self.image,
            settings.width,
            settings.height,
            settings.filter,
        )
    }
}

pub fn image_ui(_ui: &mut egui::Ui, p: &Painter, import: &mut Import, transform: &PixelTransform) {
    let ImportSettings {
        left,
        top,
        width,
        height,
        ..
    } = import.settings;
    let rect = egui::Rect::from_min_max(
        transform.screen_pos(Point::new(left, top)),
        transform.screen_pos(Point::new(left + width as i32, top + height as i32)),
    );
    let stroke = Stroke::new(1.0, IMPORT_IMAGE_EXTENTS_COLOR);
    p.rect_stroke(rect, 0.0, stroke);
    p.line_segment([rect.left_top(), rect.right_bottom()], stroke);
    p.line_segment([rect.left_bottom(), rect.right_top()], stroke);
}

/// Render the tool UI.
pub fn tool_ui(ui: &mut egui::Ui, doc: &mut Document, import: &mut Import) -> Option<Action> {
    let mut action = None;

    egui::Grid::new("import_grid").show(ui, |ui| {
        let source = &import.image;
        let target = &doc.image;
        let (source_width, source_height) = source.dimensions();
        let (target_width, target_height) = target.pixel_size();

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
            action = Some(Action::Document(DocAction::PasteTrueColor {
                source: scaled,
                target_x: import.settings.left,
                target_y: import.settings.top,
                format: import.settings.format,
            }));
        } else if ui.button("Close").clicked() {
            action = Some(Action::Ui(UiAction::SelectTool(Tool::Paint(
                Default::default(),
            ))));
        }
    });
    ui.end_row();

    action
}
