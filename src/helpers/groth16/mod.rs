pub mod prover;
pub mod verifier;


use std::cell::RefCell;
use crate::core::cs::{ConstraintSystem};
use crate::native::num::Num;
use crate::circuit::num::{CNum, Index};
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

impl<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> Clone for Groth16CS<BE, BCS> {
    fn clone(&self) -> Self {
        panic!("Clone is not implemented for Groth16CS")
    }
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

    fn enforce(&self, a:&CNum<Self>, b:&CNum<Self>, c:&CNum<Self>) {
        fn into_bellman_lc<BE:bellman::pairing::Engine, CS:ConstraintSystem>(s:&CNum<CS>) -> bellman::LinearCombination<BE> {
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
