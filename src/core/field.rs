pub use ff::{PrimeField, PrimeFieldRepr, SqrtField};
pub use ff::Field as AbstractField;

pub trait Field : PrimeField+SqrtField{
    type Repr:PrimeFieldRepr;
}

impl<T:PrimeField+SqrtField> Field for T {
    type Repr = <T as PrimeField>::Repr;
}



