//! Manage textures that display the images

use eframe::{
    egui::{Color32, TextureId},
    epi::TextureAllocator,
};
use image::imageops::FilterType;

use crate::{
    cell_image::CellImageSize, colors::TrueColor, mutation_monitor::MutationMonitor,
    ui::ViewSettings, vic::VicImage,
};

// Don't scale the texture more than this to avoid huge textures when zooming.
const MAX_SCALE: u32 = 8;

pub struct Texture {
    pub id: TextureId,
    pub settings: ViewSettings,
    pub width: usize,
    pub height: usize,
}

/// Updates the texture with the current image content, if needed.
/// Returns the texture id.
pub fn update_texture(
    image: &mut MutationMonitor<VicImage>,
    image_texture: &mut Option<Texture>,
    tex_allocator: &mut dyn TextureAllocator,
    par: f32,
    zoom: f32,
    settings: &ViewSettings,
) -> TextureId {
    let scale_x = ((par * zoom).ceil() as u32).max(1).min(MAX_SCALE);
    let scale_y = (zoom.ceil() as u32).max(1).min(MAX_SCALE);
    let (source_width, source_height) = image.size_in_pixels();
    let texture_width = source_width * scale_x as usize;
    let texture_height = source_height * scale_y as usize;

    // Recreate the texture if the size has changed or the image has been updated
    if let Some(t) = image_texture {
        if t.settings != *settings
            || t.width != texture_width
            || t.height != texture_height
            || image.dirty
        {
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
        let unscaled_image = image.render_with_settings(settings);
        let scaled_image = image::imageops::resize(
            &unscaled_image,
            unscaled_image.width() * scale_x,
            unscaled_image.height() * scale_y,
            FilterType::Nearest,
        );
        let pixels: Vec<Color32> = scaled_image
            .pixels()
            .map(|p| (<image::Rgba<u8> as Into<TrueColor>>::into(*p)).into())
            .collect();
        let texture_id =
            tex_allocator.alloc_srgba_premultiplied((texture_width, texture_height), &pixels);
        *image_texture = Some(Texture {
            id: texture_id,
            settings: settings.clone(),
            width: texture_width,
            height: texture_height,
        });
        texture_id
    };
    image.dirty = false;
    texture
}
