#[cfg(feature = "r1cs")]
mod r1cs;

#[cfg(feature = "r1cs")]
pub use r1cs::*;

#[cfg(feature = "plonk")]
mod plonk;

#[cfg(feature = "plonk")]
pub use plonk::*;



pub type Variable=usize;
pub const VARIABLE_ZERO:usize=0;



