use image::RgbaImage;
use imagequant::RGBA;

/// Returns (pixels, palette, error).
pub fn palettize<'a>(image: &RgbaImage, palette: &[RGBA]) -> (Vec<u8>, Vec<RGBA>, f64) {
    let (width, height) = image.dimensions();
    let pixels = image
        .pixels()
        .map(|p| imagequant::RGBA {
            r: p[0],
            g: p[1],
            b: p[2],
            a: p[3],
        })
        .collect::<Vec<_>>();

    let mut liq = imagequant::new();
    liq.set_speed(5);
    //liq.set_quality(0, 99);
    liq.set_max_colors(palette.len() as i32);

    let ref mut img = liq
        .new_image(&pixels, width as usize, height as usize, 0.0)
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

pub fn depalettize(width: u32, height: u32, pixels: &Vec<u8>, palette: &[RGBA]) -> RgbaImage {
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
