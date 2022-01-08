use crate::colors::TrueColor;
use image::RgbaImage;
use imgref::ImgVec;

/// Generate an image by attempting different color settings and finding the one that gives the least error.
/// Tries different character colors and finds the one that gives the least quantization error.
/// The colors in `fixed_colors` will be used in every attempt, in addition to the varying character color.
pub fn optimized_image(
    original: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    fixed_colors: &[u8],
    colors_to_attempt: impl Iterator<Item = u8>,
    palette: &[TrueColor],
) -> imgref::Img<Vec<u8>> {
    let (pixels, colors, _error) = colors_to_attempt
        .filter(|attempted_color| !fixed_colors.contains(attempted_color))
        .map(|attempted_color| {
            // Generate a list of the color combinations to try
            let mut colors = Vec::with_capacity(fixed_colors.len() + 1);
            colors.extend_from_slice(fixed_colors);
            colors.push(attempted_color);
            // Generate RGBA palette from those colors.
            let palette = colors
                .iter()
                .map(|&c| palette[c as usize])
                .collect::<Vec<_>>();
            let (pixels, error) = palettize(original, &palette);
            (pixels, colors, error)
        })
        .min_by(|(_, _, error0), (_, _, error1)| error0.partial_cmp(error1).unwrap())
        .unwrap();

    ImgVec::new(
        pixels.iter().map(|&c| colors[c as usize]).collect(),
        original.width() as usize,
        original.height() as usize,
    )
}

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
pub fn palettize(image: &RgbaImage, palette: &[TrueColor]) -> (Vec<u8>, f64) {
    use crate::colors;

    let it = image
        .pixels()
        .map(|color| colors::closest_palette_entry((*color).into(), palette.iter()))
        .map(|(index, error)| (index, error as f64));
    let indices = it.clone().map(|(index, _)| index as u8).collect();
    let error_sum = it.map(|(_, error)| error).sum();
    (indices, error_sum)
}
