use super::*;
use crate::circuit::cs::{CS, Gate};
use bellman::{ConstraintSystem, SynthesisError};
pub struct ProofRepr<E:Engine> {
    a:G1PointRepr<E>,
    b:G2PointRepr<E>,
    c:G1PointRepr<E>
}

#[repr(transparent)]
struct BellmanCS<'a, E:Engine>(&'a CS<E::Fr>);

impl<'a, E:Engine> bellman::Circuit<E::BE> for BellmanCS<'a, E> {
    fn synthesize<BCS: ConstraintSystem<E::BE>>(self, bellman_cs: &mut BCS) -> Result<(), SynthesisError> {
        let BellmanCS(cs) = self;
        let mut public_indexes = cs.public.clone();
        public_indexes.sort();
        let mut public_indexes = public_indexes.into_iter();
        let vars_length = cs.values.len();
        let mut variables = Vec::with_capacity(vars_length);
 

        let mut i = 0;
        loop {
            let t = public_indexes.next();
            for j in i..t.unwrap_or(vars_length) {
                let v = bellman_cs.alloc(
                    || format!("var_{}", j), 
                    || cs.values[j].map(|v| convert_scalar_raw(v) ).ok_or(SynthesisError::AssignmentMissing)
                ).unwrap();
                variables.push(v);
            }

            match t {
                Some(t) => {
                    let v = bellman_cs.alloc_input(
                        || format!("var_{}", t), 
                        || cs.values[t].map(|v| convert_scalar_raw(v) ).ok_or(SynthesisError::AssignmentMissing)
                    ).unwrap();
                    variables.push(v); 
                    i=t+1;
                }
                _ => { break }
            }
        }

        for (i, g) in cs.gates.iter().enumerate() {
            
        }




        Ok(())
    }
} 

pub fn prove<E:Engine, Pub:Signal<E::Fr>, Sec:Signal<E::Fr>, C: Fn(Pub::Value, Sec::Value)>(
    params:&Parameters<E>, input_pub:Pub::Value, input_sec:Sec::Value, circuit:C
) -> ProofRepr<E> {


    std::unimplemented!()
}