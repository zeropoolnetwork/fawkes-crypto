use crate::core::num::{Num, Assignment};
use bellman::{self};
use ff::{PrimeField};
use std::cell::RefCell;

use crate::core::signal::{Signal, Index};

pub trait ConstraintSystem: Sized {
    type F: PrimeField;

    fn alloc(&self, value: Option<Num<Self::F>>) -> Index;
    fn alloc_input(&self, value: Option<Num<Self::F>>) -> Index;
    fn enforce(&self, a:&Signal<Self>, b:&Signal<Self>, c:&Signal<Self>);
}

pub struct BellmanCS<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> {
    pub ninputs:RefCell<usize>,
    pub naux:RefCell<usize>,
    pub ncons:RefCell<usize>,
    pub bcs:RefCell<BCS>,
    be: std::marker::PhantomData<BE>
}

impl<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> ConstraintSystem for BellmanCS<BE, BCS> {
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
                let res = s.lc.iter().map(|(k, v)| (k, v.into_inner())).collect::<Vec<_>>();
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


impl<F:PrimeField> ConstraintSystem for TestCS<F> {
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