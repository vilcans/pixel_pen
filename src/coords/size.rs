#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size<T> {
    pub(super) width: T,
    pub(super) height: T,
}

impl<T> Size<T>
where
    T: Copy,
{
    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }
}

impl<T> Size<T>
where
    T: std::ops::Mul,
    T: Copy,
{
    /// The product width * height.
    pub fn area(&self) -> <T as std::ops::Mul>::Output {
        self.width * self.height
    }
}
