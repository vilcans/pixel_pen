use imgref::ImgVec;

use crate::vic::Char;

pub fn mirror_x(brush: &mut ImgVec<Char>) {
    for row in brush.rows_mut() {
        row.reverse();
        for c in row {
            c.mirror_x();
        }
    }
}

pub fn mirror_y(brush: &mut ImgVec<Char>) {
    let (width, height) = (brush.width(), brush.height());
    let stride = brush.stride();
    let buf = brush.buf_mut();
    for tr in 0..height / 2 {
        let br = height - 1 - tr;
        let (top, bottom) = buf.split_at_mut(br * stride);
        let t = &mut top[tr * stride..tr * stride + width];
        let b = &mut bottom[..width];
        for (a, b) in t.iter_mut().zip(b.iter_mut()) {
            std::mem::swap(a, b);
        }
    }
    for c in buf.iter_mut() {
        c.mirror_y();
    }
}
