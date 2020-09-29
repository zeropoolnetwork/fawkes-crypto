

use ff_uint::{PrimeField, PrimeFieldParams, NumRepr, Num};
use crate::core::signal::Signal;



pub mod osrng;
pub mod prover;

pub trait Engine {
    type BE: bellman::pairing::Engine;
    type Fq: PrimeField;
    type Fr: PrimeField;
}

//assuming scalar length < 128 bytes
pub fn convert_scalar_raw<Fx:PrimeField, Fy:bellman::pairing::ff::PrimeField>(from:Num<Fx>) -> Fy {
    let buff = from.as_mont_uint().as_inner().as_ref();

    let mut to = Fy::char();
    let to_ref = to.as_mut();

    assert!(buff.len()==to_ref.len());
    
    to_ref.iter_mut().zip(buff.iter()).for_each(|(a,b)| *a=*b);
    Fy::from_raw_repr(to).unwrap()
}


pub struct Parameters<E:Engine>(bellman::groth16::Parameters<E::BE>);
pub struct G1PointRepr<E:Engine>(NumRepr<<E::Fq as PrimeFieldParams>::Inner>, NumRepr<<E::Fq as PrimeFieldParams>::Inner>);
// Complex components are listed in LE notation, X+IY
pub struct G2PointRepr<E:Engine>(
    (NumRepr<<E::Fq as PrimeFieldParams>::Inner>, NumRepr<<E::Fq as PrimeFieldParams>::Inner>),
    (NumRepr<<E::Fq as PrimeFieldParams>::Inner>, NumRepr<<E::Fq as PrimeFieldParams>::Inner>)
);



pub struct VKRepr<E:Engine> {
    alpha:G1PointRepr<E>,
    beta:G2PointRepr<E>,
    gamma:G2PointRepr<E>,
    delta:G2PointRepr<E>,
    ic:Vec<G1PointRepr<E>>
}

