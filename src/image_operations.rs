use image::RgbaImage;
use rgb::RGBA8;

/// Returns (pixels, palette, error).
#[cfg(feature = "imagequant")]
pub fn palettize<'a>(image: &RgbaImage, palette: &[RGBA8]) -> (Vec<u8>, f64) {
    use rgb::AsPixels;

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
    let (final_palette, final_pixels) = res.remapped(img).unwrap();
    debug_assert_eq!(final_palette, palette);
    (final_pixels, res.quantization_error().unwrap())
}

/// Returns (pixels, palette, error).
#[cfg(not(feature = "imagequant"))]
pub fn palettize<'a>(image: &RgbaImage, palette: &[RGBA8]) -> (Vec<u8>, f64) {
    use crate::colors;

    let it = image
        .pixels()
        .map(|color| colors::closest_palette_entry(colors::to_rgba(color), palette))
        .map(|(index, error)| (index, error as f64));
    let indices = it.clone().map(|(index, _)| index).collect();
    let error_sum = it.map(|(_, error)| error).sum();
    (indices, error_sum)
}
