pub use ff::{Field as AbstractField, PrimeField, PrimeFieldRepr, SqrtField};

pub trait Field: PrimeField + SqrtField {
    type Repr: PrimeFieldRepr;
}

impl<T: PrimeField + SqrtField> Field for T {
    type Repr = <T as PrimeField>::Repr;
}
