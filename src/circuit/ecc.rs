use bellman::{
    SynthesisError,
    ConstraintSystem
};

use bellman::pairing::{
    Engine
};

use ff::{
    PrimeField
};


use super::signal::Signal;

#[derive(Clone)]
pub struct EdwardsPoint<E:Engine> {
    pub x: Signal<E>,
    pub y: Signal<E>
}

impl<E:Engine> EdwardsPoint<E> {
    pub fn double<CS:ConstraintSystem<E>>(mut cs:CS, p: &EdwardsPoint<E>) -> Result<EdwardsPoint<E>, SynthesisError>{
        let x2 = p.x.square(cs.namespace(|| "x square"))?;
        let y2 = p.y.square(cs.namespace(|| "y square"))?;
        let xy = p.x.multiply(cs.namespace(|| "xy multiply"), &p.y)?;

        let new_x = (&xy+&xy).divide(cs.namespace(|| "compute point.x"), &(&y2 - &x2))?;
        let new_y = (&x2+&y2).divide(cs.namespace(|| "compute point.y"), &(&x2 - &y2 + &Signal::Constant(E::Fr::from_str("2").unwrap())))?;

        Ok(EdwardsPoint {x: new_x, y: new_y})
    }

}