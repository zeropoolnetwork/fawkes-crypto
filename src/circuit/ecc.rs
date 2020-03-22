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
use crate::ecc::{JubJubParams};

#[derive(Clone)]
pub struct EdwardsPoint<E:Engine> {
    pub x: Signal<E>,
    pub y: Signal<E>
}

impl<E:Engine> EdwardsPoint<E> {
    pub fn alloc<CS:ConstraintSystem<E>>(mut cs:CS, p: Option<crate::ecc::EdwardsPoint<E>>) -> Result<Self, SynthesisError> {
        let (x_value, y_value) = match p {
            Some(p) => {
                let (x,y) = p.into_xy();
                (Some(x), Some(y))
            },
            None => (None, None)
        };

        let x = Signal::alloc(cs.namespace(|| "alloc x"), x_value)?;
        let y = Signal::alloc(cs.namespace(|| "alloc y"), y_value)?;
        Ok(EdwardsPoint{x, y})
    }


    pub fn double<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<EdwardsPoint<E>, SynthesisError>{
        let v = self.x.multiply(cs.namespace(|| "xy"), &self.y)?;
        let v2 = v.square(cs.namespace(|| "x^2 y^2"))?;
        let u = (&self.x+&self.y).square(cs.namespace(|| "(x+y)^2"))?;
        let new_x = (&v+&v).divide(cs.namespace(|| "compute point.x"), &(Signal::one() +  &(params.edwards_d().clone()*&v2)))?;
        let new_y = (&u-&v-&v).divide(cs.namespace(|| "compute point.x"), &(Signal::one() -  &(params.edwards_d().clone()*&v2)))?;
        Ok(EdwardsPoint {x: new_x, y: new_y})
    }

    pub fn mul_cofactor<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<EdwardsPoint<E>, SynthesisError>{
        let p2 = self.double(cs.namespace(|| "2*self"), params)?;
        let p4 = p2.double(cs.namespace(|| "4*self"), params)?;
        let p8 = p4.double(cs.namespace(|| "8*self"), params)?;
        Ok(p8)
    }



    pub fn add<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, p: &EdwardsPoint<E>, params: &J) -> Result<EdwardsPoint<E>, SynthesisError> {
        let v1 = self.x.multiply(cs.namespace(|| "x1y2"), &p.y)?;
        let v2 = p.x.multiply(cs.namespace(|| "x2y1"), &self.y)?;
        let v12 = v1.multiply(cs.namespace(|| "x1y2x2y1"), &v2)?;
        let u = (&self.x+&self.y).multiply(cs.namespace(|| "(x1+y1)*(x2+y2)"), &(&p.x+&p.y))?;
        let new_x = (&v1+&v2).divide(cs.namespace(|| "compute point.x"), &(Signal::one() +  &(params.edwards_d().clone()*&v12)))?;
        let new_y = (&u-&v1-&v2).divide(cs.namespace(|| "compute point.x"), &(Signal::one() -  &(params.edwards_d().clone()*&v12)))?;
        Ok(EdwardsPoint {x: new_x, y: new_y})
    }

    pub fn assert_in_curve<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<(), SynthesisError> {
        let x2 = self.x.square(cs.namespace(|| "x^2"))?;
        let y2 = self.y.square(cs.namespace(|| "y^2"))?;
        cs.enforce(|| "point should be on curve", |_| y2.lc(), |zero| zero + CS::one() - (params.edwards_d().clone(), &y2.lc()), |zero| zero + CS::one() + &x2.lc());
        Ok(())
    }

    pub fn assert_in_subgroup<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<(), SynthesisError> {
        let preimage_value = match (self.x.get_value(), self.y.get_value()) {
            (Some(x), Some(y)) => {
                let p = crate::ecc::EdwardsPoint::from_xy_unchecked(x, y);
                Some(p.mul(params.edwards_inv_cofactor().into_repr(), params))
            },
            _ => None
        };

        let preimage = EdwardsPoint::alloc(cs.namespace(|| "alloc preimage point"), preimage_value)?;
        let preimage8 = preimage.mul_cofactor(cs.namespace(|| "8*preimage"), params)?;

        (&self.x - &preimage8.x).assert_zero(cs.namespace(|| "assert x equality"))?;
        (&self.y - &preimage8.y).assert_zero(cs.namespace(|| "assert y equality"))?;
        
        Ok(())
        
    }

}


