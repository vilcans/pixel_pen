use eframe::egui::{Color32, Painter, Rect, Shape, Vec2};

pub fn palette_patch(
    painter: &Painter,
    rect: &Rect,
    color: Color32,
    selected_background: bool,
    selected_border: bool,
    selected_aux: bool,
    selected_pen: bool,
) {
    let size = rect.width();
    let d = size * 0.2;
    let r = d / 2.0;
    let icon_distance = d * 1.1;
    let number_of_icons = if selected_background { 1 } else { 0 }
        + if selected_border { 1 } else { 0 }
        + if selected_aux { 1 } else { 0 };
    let mut icon_num = 0;
    let mut next_icon_pos = || {
        let i = icon_num;
        icon_num += 1;
        let xoffs = (i as f32 - (number_of_icons - 1) as f32 / 2.0) * icon_distance;
        Rect::from_center_size(rect.center_bottom() + Vec2::new(xoffs, -r), Vec2::new(d, d))
    };

    // The patch
    let patch_rect = Rect::from_min_size(
        rect.left_top() + Vec2::new(0.0, d),
        rect.size() + Vec2::new(0.0, -d * 2.2),
    );
    if selected_pen {
        painter.rect_filled(patch_rect, size * 0.1, color);
    } else {
        painter.rect_filled(patch_rect.shrink(size * 0.05), size * 0.1, color);
    }

    if selected_pen {
        painter.add(Shape::polygon(
            vec![
                patch_rect.center_top(),
                rect.center_top() - Vec2::new(r, 0.0),
                rect.center_top() + Vec2::new(r, 0.0),
            ],
            Color32::WHITE,
            (0.0, Color32::WHITE),
        ));
    }
    if selected_background {
        painter.rect_filled(next_icon_pos(), 0.0, Color32::WHITE);
    }
    if selected_border {
        let width = size * 0.04;
        painter.rect_stroke(
            next_icon_pos().shrink(width / 2.0),
            0.0,
            (width, Color32::WHITE),
        );
    }
    if selected_aux {
        let rect = next_icon_pos();
        painter.circle_filled(rect.center(), rect.width() / 2.0, Color32::WHITE);
    }
}
