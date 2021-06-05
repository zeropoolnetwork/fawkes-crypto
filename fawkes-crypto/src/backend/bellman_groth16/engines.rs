use super::*;

pub struct Bn256;
pub struct Bls12_381;

pub trait Engine {
    type BE: bellman::pairing::Engine;
    type Fq: PrimeField;
    type Fr: PrimeField;
}

impl Engine for Bn256 {
    type BE = bellman::pairing::bn256::Bn256;
    type Fq = crate::engines::bn256::Fq;
    type Fr = crate::engines::bn256::Fr;
}

impl Engine for Bls12_381 {
    type BE = bellman::pairing::bls12_381::Bls12;
    type Fq = crate::engines::bls12_381::Fq;
    type Fr = crate::engines::bls12_381::Fr;
}
