use ff::{SqrtField, PrimeField};

pub trait PrimeFieldEx : PrimeField + SqrtField {}