use core::fmt::Display;

#[derive(Clone, Copy, Debug)]
pub enum ButtonDirection {
    Left,
    Right,
}

impl Display for ButtonDirection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}