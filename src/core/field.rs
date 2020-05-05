pub use ff::{PrimeField, PrimeFieldRepr, SqrtField};
pub use ff::Field as AbstractField;

pub trait Field : PrimeField+SqrtField{
    type Repr:PrimeFieldRepr;
}

impl<T:PrimeField+SqrtField> Field for T {
    type Repr = <T as PrimeField>::Repr;
}


#[derive(PrimeField)]
#[PrimeFieldModulus = "2736030358979909402780800718157159386076813972158567259200215660948447373041"]
#[PrimeFieldGenerator = "7"]
pub struct Fs(FsRepr);

