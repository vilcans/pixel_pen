//! Some functionality for importing images.

use crate::error::Error;
use crate::vic::ColorFormat;
use image::imageops::FilterType;
use image::DynamicImage;
use image::GenericImageView;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "FilterType")]
enum FilterTypeForSerialization {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Debug, Clone)]
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
