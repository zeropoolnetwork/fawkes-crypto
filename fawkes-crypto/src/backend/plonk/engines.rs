use super::*;

pub struct Bn256;
pub struct Bls12_381;

pub trait Engine {
    type BE: halo2_curves::pairing::MultiMillerLoop+std::fmt::Debug;
    type Fq: PrimeField;
    type Fr: PrimeField;
}

impl Engine for Bn256 {
    type BE = halo2_curves::bn256::Bn256;
    type Fq = crate::engines::bn256::Fq;
    type Fr = crate::engines::bn256::Fr;
}

