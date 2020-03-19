pub mod signal;
pub mod bitify;
pub mod poseidon;
pub mod ecc;

use bellman_ce::{
    SynthesisError
};


pub trait Assignment<T> {
    fn get(&self) -> Result<&T, SynthesisError>;
    fn grab(self) -> Result<T, SynthesisError>;
}

impl<T: Clone> Assignment<T> for Option<T> {
    fn get(&self) -> Result<&T, SynthesisError> {
        match *self {
            Some(ref v) => Ok(v),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }

    fn grab(self) -> Result<T, SynthesisError> {
        match self {
            Some(v) => Ok(v),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }
}