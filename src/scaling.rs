use imgref::{ImgRef, ImgVec};

pub fn scale_image<Pixel>(source: ImgRef<'_, Pixel>, scale_x: f32, scale_y: f32) -> ImgVec<Pixel>
where
    Pixel: Copy,
{
    let source_w = source.width();
    let source_h = source.height();
    let target_w = (source_w as f32 * scale_x).round() as u32;
    let target_h = (source_h as f32 * scale_y).round() as u32;

    let pixels = (0..target_h as u32)
        .flat_map(move |y| {
            let source_row = *source
                .sub_image(0, (y as f32 / scale_y) as usize, source_w, 1)
                .buf();
            (0..target_w).map(move |x| source_row[(x as f32 / scale_x) as usize])
        })
        .collect::<Vec<_>>();

    ImgVec::new(pixels, target_w as usize, target_h as usize)
}
