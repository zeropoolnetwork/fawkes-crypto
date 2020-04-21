use std::cell::RefCell;
use crate::core::cs::{ConstraintSystem, Circuit};
use crate::core::num::Num;
use crate::core::signal::{Signal, Index};
use crate::core::osrng::OsRng;
use ff::Field;

use bellman::{self, SynthesisError};


pub trait Assignment<T> {
    fn get(&self) -> Result<&T, SynthesisError>;
    fn grab(self) -> Result<T, SynthesisError>;
}

impl<T: Clone> Assignment<T> for Option<T> {
    fn get(&self) -> Result<&T, SynthesisError> {
        match *self {
            Some(ref v) => Ok(v),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }

    fn grab(self) -> Result<T, SynthesisError> {
        match self {
            Some(v) => Ok(v),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }
}

impl<T: Field> Assignment<T> for Option<Num<T>> {
    fn get(&self) -> Result<&T, SynthesisError> {
        match self {
            Some(ref v) => Ok(&v.0),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }

    fn grab(self) -> Result<T, SynthesisError> {
        match self {
            Some(v) => Ok(v.into_inner()),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }
}



pub struct Groth16CS<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> {
    pub ninputs:RefCell<usize>,
    pub naux:RefCell<usize>,
    pub ncons:RefCell<usize>,
    pub bcs:RefCell<BCS>,
    be: std::marker::PhantomData<BE>
}

impl<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>>  Groth16CS<BE, BCS> {
    pub fn new(cs:BCS) -> Self {
        Self {
            ninputs: RefCell::new(1),
            naux: RefCell::new(0),
            ncons: RefCell::new(0),
            bcs: RefCell::new(cs),
            be: std::marker::PhantomData
        }

    }
}


impl<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> ConstraintSystem for Groth16CS<BE, BCS> {
    type F = BE::Fr;

    fn alloc(&self, value: Option<Num<Self::F>>) -> Index {
        let mut naux_ref = self.ninputs.borrow_mut();
        let naux = *naux_ref;
        *naux_ref+=1;
        self.bcs.borrow_mut().alloc(||format!("a[{}]", naux), || value.grab()).map(|e| unsafe{std::mem::transmute(e)}).unwrap()
            
    }
    fn alloc_input(&self, value: Option<Num<Self::F>>) -> Index {
        let mut ninputs_ref = self.ninputs.borrow_mut();
        let ninputs = *ninputs_ref;
        *ninputs_ref+=1;
        self.bcs.borrow_mut().alloc_input(||format!("i[{}]", ninputs), || value.grab()).map(|e| unsafe{std::mem::transmute(e)}).unwrap()
    }

    fn enforce(&self, a:&Signal<Self>, b:&Signal<Self>, c:&Signal<Self>) {
        fn into_bellman_lc<BE:bellman::pairing::Engine, CS:ConstraintSystem>(s:&Signal<CS>) -> bellman::LinearCombination<BE> {
                let res = s.lc.iter().map(|(k, v)| (*k, v.into_inner())).collect::<Vec<_>>();
                unsafe {std::mem::transmute(res)}
        }
        
        let mut ncons_ref = self.ncons.borrow_mut();
        let ncons = *ncons_ref;
        *ncons_ref += 1;
        let a = into_bellman_lc(a);
        let b = into_bellman_lc(b);
        let c = into_bellman_lc(c);
        self.bcs.borrow_mut().enforce(|| format!("c[{}]", ncons), |_| a, |_| b, |_| c);
    }
}

struct HelperCircuit<'a, C:Circuit>(pub &'a C);

impl <'a, BE:bellman::pairing::Engine, C:Circuit<F=BE::Fr>> bellman::Circuit<BE> for HelperCircuit<'a, C> {
    fn synthesize<CS: bellman::ConstraintSystem<BE>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let ref cs = Groth16CS::new(cs.namespace(|| "root"));
        self.0.synthesize(cs);
        Ok(())
    }
}

pub fn groth16_generate_keys<BE:bellman::pairing::Engine, C:Circuit<F=BE::Fr>+Default>() -> bellman::groth16::Parameters<BE> {
    let ref mut rng = OsRng::new();
    let c = C::default();
    bellman::groth16::generate_random_parameters(HelperCircuit(&c), rng).unwrap()
}

pub fn groth16_proof<BE:bellman::pairing::Engine, C:Circuit<F=BE::Fr>>(c:&C, params:&bellman::groth16::Parameters<BE>) -> bellman::groth16::Proof<BE>{
    let ref mut rng = OsRng::new();
    bellman::groth16::create_random_proof(HelperCircuit(c), params, rng).unwrap()
}

#[cfg(test)]
mod bellman_test {
    use super::*;
    use ff::{PrimeField, SqrtField};
    use bellman::pairing::bn256::{Fr, Bn256};
    use crate::core::signal::{Signal, AbstractSignal};
    use crate::native::poseidon::PoseidonParams;
    use crate::circuit::poseidon::c_poseidon;
    use rand::{Rng, thread_rng};


    struct CheckPreimageKnowledge<F:PrimeField+SqrtField> {
        image:Option<Num<F>>,
        preimage:Option<Num<F>>
    }

    impl<F:PrimeField+SqrtField> Default for CheckPreimageKnowledge<F> {
        fn default() -> Self {
            Self {
                image: None,
                preimage: None
            }
        }
    }

    impl<F:PrimeField+SqrtField> Circuit for CheckPreimageKnowledge<F> {
        type F = F;
        fn synthesize<CS: ConstraintSystem<F=F>>(
            &self,
            cs: &CS
        ) {
            let image = Signal::alloc(cs, self.image);
            image.inputize();
            let preimage = Signal::alloc(cs, self.preimage);
            let ref poseidon_params = PoseidonParams::<F>::new(2, 8, 53);
            let image_computed = c_poseidon([preimage].as_ref(), poseidon_params);
            (&image-&image_computed).assert_zero();
        }
    }


    #[test]
    fn test_helper() {
        let mut rng = thread_rng();
        let params = groth16_generate_keys::<Bn256, CheckPreimageKnowledge<Fr>>();
        let preimage = rng.gen();
        let ref poseidon_params = PoseidonParams::<Fr>::new(2, 8, 53);
        let image = crate::native::poseidon::poseidon([preimage].as_ref(), poseidon_params);
        let c = CheckPreimageKnowledge {image:Some(image), preimage:Some(preimage)};
        let proof = groth16_proof(&c, &params);

        let pvk = bellman::groth16::prepare_verifying_key(&params.vk);
        let res = bellman::groth16::verify_proof(&pvk, &proof, [image.into_inner()].as_ref()).unwrap();
        assert!(res, "proof should be valid");
    }


}
