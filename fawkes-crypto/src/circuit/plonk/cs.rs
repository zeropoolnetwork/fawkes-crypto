use crate::{
    circuit::num::CNum,
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
};

use std::{cell::RefCell, rc::Rc};

pub type RCS<Fr> = Rc<RefCell<CS<Fr>>>;

#[derive(Clone, Debug)]
pub enum Gate<Fr: PrimeField> {
    // a*x + b *y + c*z + d*x*y + e == 0
    Arith(
        Num<Fr>,
        usize,
        Num<Fr>,
        usize,
        Num<Fr>,
        usize,
        Num<Fr>,
        Num<Fr>,
    ),
}
#[derive(Clone, Debug)]
pub struct CS<Fr: PrimeField> {
    pub values: Vec<Option<Num<Fr>>>,
    pub gates: Vec<Gate<Fr>>,
    pub tracking: bool,
    pub public: Vec<usize>,
}

impl<Fr: PrimeField> CS<Fr> {
    pub fn num_constraints(&self) -> usize {
        self.gates.len()
    }

    pub fn new(tracking: bool) -> Self {
        Self {
            values: vec![],
            gates: vec![],
            tracking,
            public: vec![],
        }
    }

    pub fn rc_new(tracking: bool) -> RCS<Fr> {
        Rc::new(RefCell::new(Self::new(tracking)))
    }

    // a*b === c
    pub fn enforce_mul(a: &CNum<Fr>, b: &CNum<Fr>, c: &CNum<Fr>) {
        let mut rcs = a.get_cs().borrow_mut();
        if rcs.tracking {
            match (a.value, b.value, c.value) {
                (Some(a), Some(b), Some(c)) => {
                    assert!(a * b == c, "Not satisfied constraint");
                }
                _ => {}
            }
        }
        rcs.gates.push(Gate::Arith(
            a.lc.0 * b.lc.2,
            a.lc.1,
            a.lc.2 * b.lc.0,
            b.lc.1,
            -c.lc.0,
            c.lc.1,
            a.lc.0 * b.lc.0,
            a.lc.2 * b.lc.2 - c.lc.2,
        ))
    }

    pub fn enforce_add(a: &CNum<Fr>, b: &CNum<Fr>, c: &CNum<Fr>) {
        let mut rcs = a.get_cs().borrow_mut();
        if rcs.tracking {
            match (a.value, b.value, c.value) {
                (Some(a), Some(b), Some(c)) => {
                    assert!(a + b == c, "Not satisfied constraint");
                }
                _ => {}
            }
        }
        rcs.gates.push(Gate::Arith(
            a.lc.0,
            a.lc.1,
            b.lc.0,
            b.lc.1,
            -c.lc.0,
            c.lc.1,
            Num::ZERO,
            a.lc.2 + b.lc.2 - c.lc.2,
        ))
    }

    pub fn enforce_pub(n: &CNum<Fr>) {
        let v = if n.lc.0 == Num::ONE && n.lc.2 == Num::ZERO {
            n.lc.1
        } else {
            let m: CNum<Fr> = n.derive_alloc(n.value.as_ref());
            m.assert_eq(n);
            m.lc.1
        };

        n.get_cs().borrow_mut().public.push(v);
    }

    pub fn alloc(cs: &RCS<Fr>, value: Option<&Num<Fr>>) -> CNum<Fr> {
        let mut rcs = cs.borrow_mut();
        let n_vars = rcs.values.len();
        let v = n_vars;
        rcs.values.push(value.cloned());
        CNum {
            value: value.cloned(),
            lc: (Num::ONE, v, Num::ZERO),
            cs: cs.clone(),
        }
    }
}
