use crate::core::num::{Num};
use ff::{PrimeField, SqrtField};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::signal::{Signal, Index};

pub trait ConstraintSystem: Sized+Clone {
    type F: PrimeField+SqrtField;

    fn alloc(&self, value: Option<Num<Self::F>>) -> Index;
    fn alloc_input(&self, value: Option<Num<Self::F>>) -> Index;
    fn enforce(&self, a:&Signal<Self>, b:&Signal<Self>, c:&Signal<Self>);
}


pub trait Circuit {
    type F: PrimeField+SqrtField;

    fn synthesize<CS: ConstraintSystem<F=Self::F>>(
        &self,
        cs: &CS
    );
}


pub struct TestCS<F:PrimeField> {
    pub ninputs:RefCell<usize>,
    pub naux:RefCell<usize>,
    pub ncons:RefCell<usize>,
    pub variables:RefCell<HashMap<Index, Num<F>>>,
    f: std::marker::PhantomData<F>
}

impl<F:PrimeField> TestCS<F> {
    pub fn new() -> Self {
        Self {
            ninputs: RefCell::new(1),
            naux: RefCell::new(0),
            ncons: RefCell::new(0),
            variables: RefCell::new(HashMap::new()),
            f: std::marker::PhantomData
        }
    }

    pub fn num_constraints(&self) -> usize {
        *self.ncons.borrow()
    }
}

impl<F:PrimeField+SqrtField> Clone for TestCS<F> {
    fn clone(&self) -> Self {
        panic!("Clone is not implemented for TestCS")
    }
}



impl<F:PrimeField+SqrtField> ConstraintSystem for TestCS<F> {
    type F = F;

    fn alloc(&self, v: Option<Num<Self::F>>) -> Index {
        let mut naux_ref = self.ninputs.borrow_mut();
        let naux = *naux_ref;
        *naux_ref+=1;
        let index = Index::Input(naux);
        if v.is_some() {
            (*self.variables.borrow_mut()).insert(index, v.unwrap());
        }
        index
    }
    fn alloc_input(&self, v: Option<Num<Self::F>>) -> Index {
        let mut ninputs_ref = self.ninputs.borrow_mut();
        let ninputs = *ninputs_ref;
        *ninputs_ref+=1;
        let index = Index::Input(ninputs);
        if v.is_some() {
            (*self.variables.borrow_mut()).insert(index, v.unwrap());
        }
        index
    }

    fn enforce(&self, a:&Signal<Self>, b:&Signal<Self>, c:&Signal<Self>) {
        *self.ncons.borrow_mut() += 1;
        match (a.value, b.value, c.value) {
            (Some(a), Some(b), Some(c)) => {
                assert!(a*b==c, "Not satisfied constraint");
            },
            _ => {}
        }
    }
}