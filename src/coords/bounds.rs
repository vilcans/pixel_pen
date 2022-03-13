use std::ops::Deref;

use euclid::{num::Zero, Point2D, Rect, Size2D};

use super::CellPos;

/// Checks whether a position is within the given bounds.
/// Returns `Some(WithinBounds<_>)` if it is, otherwise `None`.
pub fn within_bounds<T, U>(
    candidate: Point2D<T, U>,
    bounds: Size2D<T, U>,
) -> Option<WithinBounds<Point2D<T, U>>>
where
    T: Zero + Copy + std::ops::Add<Output = T> + PartialOrd,
{
    let bounds = Rect::new(Point2D::zero(), (bounds.width, bounds.height).into());
    if bounds.contains(candidate) {
        Some(WithinBounds(candidate))
    } else {
        None
    }
}

/// Checks that this rectangle fits inside a certain size.
/// If it does, returns a `WithinBounds<_>`, otherwise `None`.
pub fn rect_within_size<T, U>(
    candidate: Rect<T, U>,
    bounds: Size2D<T, U>,
) -> Option<WithinBounds<Rect<T, U>>>
where
    T: Zero + Copy + std::ops::Add<Output = T> + PartialOrd,
{
    let bounds = Rect::new(Point2D::zero(), (bounds.width, bounds.height).into());
    if bounds.contains_rect(&candidate) {
        Some(WithinBounds(candidate))
    } else {
        None
    }
}

pub fn clamp_rect_to_bounds<T, U>(
    candidate: Rect<T, U>,
    bounds: Size2D<T, U>,
) -> WithinBounds<Rect<T, U>>
where
    T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + Ord + Zero,
{
    let left = candidate.min_x().clamp(T::zero(), bounds.width);
    let right = candidate.max_x().clamp(T::zero(), bounds.width);
    let top = candidate.min_y().clamp(T::zero(), bounds.height);
    let bottom = candidate.max_y().clamp(T::zero(), bounds.height);
    WithinBounds::assume_within_bounds(Rect::new(
        Point2D::new(left, top),
        Size2D::new(right - left, bottom - top),
    ))
}

/// Wraps a value that is known to be within bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WithinBounds<T>(T);

impl<T> WithinBounds<T> {
    pub fn assume_within_bounds(t: T) -> WithinBounds<T> {
        Self(t)
    }
}

impl<T> Deref for WithinBounds<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl WithinBounds<CellPos> {
    /// Get the horizontal position (column) and vertical position (row) as a tuple.
    pub fn as_tuple(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}

#[cfg(test)]
mod test {
    struct TestUnit;
    type TestRect = euclid::Rect<i32, TestUnit>;
    type TestPos = euclid::Point2D<i32, TestUnit>;
    type TestSize = euclid::Size2D<i32, TestUnit>;

    use super::{clamp_rect_to_bounds, within_bounds};

    #[test]
    fn within_bounds_test() {
        let c = TestPos::new(1, 2);
        let v = within_bounds(c, TestSize::new(10, 20));
        assert!(v.is_some());
    }

    #[test]
    fn within_bounds_successful() {
        let c = TestPos::new(10, 21);
        let v = within_bounds(c, TestSize::new(10, 20));
        assert!(v.is_none());
    }

    #[test]
    fn add_pos_and_size() {
        let c = TestPos::new(5, 7);
        let s = TestSize::new(10, 100);
        assert_eq!(TestPos::new(15, 107), c + s);
    }

    #[test]
    fn rect_clamp_to_size() {
        let r = TestRect::new(TestPos::new(2, 10), TestSize::new(8, 4));
        let c = clamp_rect_to_bounds(r, TestSize::new(5, 12));
        assert_eq!(*c, TestRect::new(TestPos::new(2, 10), TestSize::new(3, 2)));
    }
}
