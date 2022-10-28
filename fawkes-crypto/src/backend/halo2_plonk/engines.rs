use ff_uint::PrimeField;

pub struct Bn256;

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
