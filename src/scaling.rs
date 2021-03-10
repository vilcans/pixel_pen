use imgref::{ImgRef, ImgVec};

pub fn scale_image<Pixel>(source: ImgRef<'_, Pixel>, scale_x: u32, scale_y: u32) -> ImgVec<Pixel>
where
    Pixel: Copy + Default,
{
    let source_w = source.width() as u32;
    let source_h = source.height() as u32;
    let target_w = source_w * scale_x;
    let target_h = source_h * scale_y;

    let mut pixels = vec![Pixel::default(); (target_w * target_h) as usize];
    for (sy, source_row) in source.rows().enumerate() {
        let dy = sy * scale_y as usize;
        let first_row_range = dy * target_w as usize..(dy + 1) * target_w as usize;
        scale_row(source_row, &mut pixels[first_row_range.clone()], scale_x);
        for dy in dy + 1..dy + scale_y as usize {
            pixels.copy_within(first_row_range.clone(), dy * target_w as usize)
        }
    }

    ImgVec::new(pixels, target_w as usize, target_h as usize)
}

fn scale_row<T>(source: &[T], target: &mut [T], scale: u32)
where
    T: Copy,
{
    debug_assert_eq!(source.len() * scale as usize, target.len());
    let mut i = 0usize;
    for &pixel in source {
        for _ in 0..scale {
            target[i] = pixel;
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale() {
        let source = ImgVec::new(vec![1, 2, 3, 4, 5, 6], 3, 2);
        let result = scale_image(source.as_ref(), 4, 3);
        let (buf, w, h) = result.into_contiguous_buf();
        assert_eq!(3 * 4, w);
        assert_eq!(2 * 3, h);
        #[rustfmt::skip]
        assert_eq!(buf, vec![
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3,
            4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6,
            4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6,
            4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6,
        ]);
    }
}
