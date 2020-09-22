use ff_uint::{Num, PrimeField};
use crate::circuit::plonk::{num::CNum, cs::CS};
use crate::circuit::general::traits::{signal::Signal, bool::SignalBool};

#[derive(Clone, Debug)]
pub struct CBool<Fr:PrimeField>(CNum<Fr>);

impl<Fr:PrimeField> SignalBool for CBool<Fr> {
    type Num=CNum<Fr>;
} 

impl<Fr:PrimeField> CBool<Fr> {
    pub fn new_unchecked(n:&CNum<Fr>) -> Self {
        CBool(n.clone())
    }

    pub fn new(n: &CNum<Fr>) -> Self {
        n.assert_bit();
        Self::new_unchecked(n)
    }

    pub fn to_num(&self) -> CNum<Fr> {
        self.0.clone()
    }
}
