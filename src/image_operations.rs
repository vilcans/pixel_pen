use image::RgbaImage;
use rgb::{AsPixels, RGBA8};

/// Returns (pixels, palette, error).
pub fn palettize<'a>(image: &RgbaImage, palette: &[RGBA8]) -> (Vec<u8>, Vec<RGBA8>, f64) {
    let mut liq = imagequant::new();
    liq.set_speed(5);
    //liq.set_quality(0, 99);
    liq.set_max_colors(palette.len() as i32);

    let (width, height) = image.dimensions();
    let ref mut img = liq
        .new_image(
            image.as_raw().as_pixels(),
            width as usize,
            height as usize,
            0.0,
        )
        .unwrap();
    for &palette_entry in palette {
        img.add_fixed_color(palette_entry);
    }

    let mut res = match liq.quantize(img) {
        Ok(res) => res,
        Err(err) => panic!("Quantization failed, because: {:?}", err),
    };

    // Enable dithering for subsequent remappings
    res.set_dithering_level(1.0);

    // You can reuse the result to generate several images with the same palette
    let (palette, final_pixels) = res.remapped(img).unwrap();
    (final_pixels, palette, res.quantization_error().unwrap())
}

pub fn depalettize(width: u32, height: u32, pixels: &Vec<u8>, palette: &[RGBA8]) -> RgbaImage {
    let mut rgba_bytes = Vec::with_capacity((width * height * 4) as usize);
    for &palette_index in pixels {
        let rgba = palette[palette_index as usize];
        rgba_bytes.push(rgba.r);
        rgba_bytes.push(rgba.g);
        rgba_bytes.push(rgba.b);
        rgba_bytes.push(rgba.a);
    }
    RgbaImage::from_vec(width, height, rgba_bytes).unwrap()
}
