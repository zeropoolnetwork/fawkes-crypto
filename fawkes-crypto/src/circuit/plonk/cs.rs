use ff_uint::{Num, PrimeField};
use crate::circuit::general::Variable;
use crate::circuit::general::traits::signal::Signal;



#[derive(Clone, Debug)]
pub enum Gate<Fr:PrimeField> {
    Pub(Variable),
    // a*x + b *y + c*z + d*x*y + e == 0
    Arith(Num<Fr>, Variable, Num<Fr>, Variable, Num<Fr>, Variable, Num<Fr>, Num<Fr>)
}
#[derive(Clone, Debug)]
pub struct CS<Fr:PrimeField> {
    pub n_vars: usize,
    pub gates: Vec<Gate<Fr>>
}
