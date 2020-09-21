pub mod traits;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Variable(pub usize);

impl Variable {
    pub const ZERO:Self = Self(0);
}
