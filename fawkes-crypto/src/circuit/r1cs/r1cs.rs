use ff_uint::PrimeField;
use crate::circuit::Variable;

pub struct LC<Fr:PrimeField>(pub Fr, pub Vec<(Fr, Variable)>);

pub enum Gate<Fr:PrimeField> {
    Pub(Variable),
    Con(LC<Fr>, LC<Fr>, LC<Fr>)
}

pub struct CS<Fr:PrimeField>(Vec<Gate<Fr>>);

