use crate::colors::TrueColor;
use image::RgbaImage;

/// Returns (pixels, palette, error).
#[cfg(feature = "imagequant")]
pub fn palettize(image: &RgbaImage, palette: &[TrueColor]) -> (Vec<u8>, f64) {
    use rgb::AsPixels;

    let mut liq = imagequant::new();
    liq.set_speed(5);
    //liq.set_quality(0, 99);
    liq.set_max_colors(palette.len() as i32);

    let (width, height) = image.dimensions();
    let mut img = liq
        .new_image(
            image.as_raw().as_pixels(),
            width as usize,
            height as usize,
            0.0,
        )
        .unwrap();
    for palette_entry in palette {
        img.add_fixed_color((*palette_entry).into());
    }

    let mut res = match liq.quantize(&img) {
        Ok(res) => res,
        Err(err) => panic!("Quantization failed, because: {:?}", err),
    };

    // Enable dithering for subsequent remappings
    res.set_dithering_level(1.0);

    // You can reuse the result to generate several images with the same palette
    let (_final_palette, final_pixels) = res.remapped(&mut img).unwrap();
    //debug_assert_eq!(final_palette, palette);
    (final_pixels, res.quantization_error().unwrap())
}

/// Returns (pixels, palette, error).
#[cfg(not(feature = "imagequant"))]
pub fn palettize(image: &RgbaImage, palette: &[RGBA8]) -> (Vec<u8>, f64) {
    use crate::colors;

    let it = image
        .pixels()
        .map(|color| colors::closest_palette_entry(colors::rgba_from_image(*color), palette.iter()))
        .map(|(index, error)| (index, error as f64));
    let indices = it.clone().map(|(index, _)| index as u8).collect();
    let error_sum = it.map(|(_, error)| error).sum();
    (indices, error_sum)
}
