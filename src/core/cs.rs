use crate::core::num::{Num};
use ff::{PrimeField, SqrtField};
use std::cell::RefCell;

use crate::core::signal::{Signal, Index};

pub trait ConstraintSystem: Sized {
    type F: PrimeField+SqrtField;

    fn alloc(&self, value: Option<Num<Self::F>>) -> Index;
    fn alloc_input(&self, value: Option<Num<Self::F>>) -> Index;
    fn enforce(&self, a:&Signal<Self>, b:&Signal<Self>, c:&Signal<Self>);
}


pub struct TestCS<F:PrimeField> {
    pub ninputs:RefCell<usize>,
    pub naux:RefCell<usize>,
    pub ncons:RefCell<usize>,
    f: std::marker::PhantomData<F>
}

impl<F:PrimeField> TestCS<F> {
    pub fn new() -> Self {
        Self {
            ninputs: RefCell::new(1),
            naux: RefCell::new(0),
            ncons: RefCell::new(0),
            f: std::marker::PhantomData
        }
    }

    pub fn num_constraints(&self) -> usize {
        *self.ncons.borrow()
    }
}


impl<F:PrimeField+SqrtField> ConstraintSystem for TestCS<F> {
    type F = F;

    fn alloc(&self, _: Option<Num<Self::F>>) -> Index {
        let mut naux_ref = self.ninputs.borrow_mut();
        let naux = *naux_ref;
        *naux_ref+=1;
        Index::Input(naux)
    }
    fn alloc_input(&self, _: Option<Num<Self::F>>) -> Index {
        let mut ninputs_ref = self.ninputs.borrow_mut();
        let ninputs = *ninputs_ref;
        *ninputs_ref+=1;
        Index::Input(ninputs)
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