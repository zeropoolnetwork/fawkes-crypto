use crate::{
    circuit::num::CNum,
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
};

use std::{cell::RefCell, rc::Rc};

pub type RCS<C> = Rc<RefCell<C>>;

/// A `Gate` describes constraint of the form
///
/// ```ignore
/// a*x + b*y + c*z + d*x*y + e == 0
/// ```
///
/// where `x`, `y`, `z` are variable witness elements (represented here as
/// indices), while the `a` ... `e` values are concrete constants represented
/// here as field values.
#[derive(Clone, Debug)]
pub struct Gate<Fr: PrimeField> {
    pub a: Num<Fr>,
    pub x: usize,
    pub b: Num<Fr>,
    pub y: usize,
    pub c: Num<Fr>,
    pub z: usize,
    pub d: Num<Fr>,
    pub e: Num<Fr>,
}

pub trait CS: Clone {
    type Fr: PrimeField;
    type GateIterator: Iterator<Item=Gate<Self::Fr>>;

    fn num_gates(&self) -> usize;
    fn num_input(&self) -> usize;
    fn num_aux(&self) -> usize;
    fn get_value(&self, index: usize) -> Option<Num<Self::Fr>>;
    fn get_gate_iterator(&self) -> Self::GateIterator;

    fn as_public(&self) -> &[usize];

    // a * b == c
    fn enforce_mul(a: &CNum<Self>, b: &CNum<Self>, c: &CNum<Self>);

    // a + b == c
    fn enforce_add(a: &CNum<Self>, b: &CNum<Self>, c: &CNum<Self>);

    fn inputize(n: &CNum<Self>);
    fn alloc(cs: &RCS<Self>, value: Option<&Num<Self::Fr>>) -> CNum<Self>;

    fn const_tracker_before(&mut self) -> Option<bool> {
        None
    }

    fn const_tracker_after(&mut self, _:bool) {}
}

#[derive(Clone, Debug)]
pub struct BuildCS<Fr: PrimeField> {
    pub values: Vec<Option<Num<Fr>>>,
    pub gates: Vec<Gate<Fr>>,
    pub tracking: bool,
    pub public: Vec<usize>,
}

impl<Fr: PrimeField> BuildCS<Fr> {
    pub fn new(tracking: bool) -> Self {
        Self {
            values: vec![],
            gates: vec![],
            tracking,
            public: vec![],
        }
    }

    pub fn rc_new(tracking: bool) -> RCS<Self> {
        Rc::new(RefCell::new(Self::new(tracking)))
    }
}

impl<Fr: PrimeField> CS for BuildCS<Fr> {
    type Fr = Fr;
    type GateIterator = std::vec::IntoIter<Gate<Self::Fr>>;

    fn num_gates(&self) -> usize {
        self.gates.len()
    }

    fn num_input(&self) -> usize {
        self.public.len()
    }

    fn num_aux(&self) -> usize {
        self.values.len() - self.public.len()
    }

    fn get_value(&self, index: usize) -> Option<Num<Self::Fr>> {
        self.values[index]
    }

    fn get_gate_iterator(&self) -> Self::GateIterator {
        self.gates.clone().into_iter()
    }

    fn as_public(&self) -> &[usize] {
        &self.public
    }

    // a*b === c
    fn enforce_mul(x: &CNum<Self>, y: &CNum<Self>, z: &CNum<Self>) {
        let mut rcs = x.get_cs().borrow_mut();
        if rcs.tracking {
            match (x.value, y.value, z.value) {
                (Some(x), Some(y), Some(z)) => {
                    assert!(x * y == z, "Not satisfied constraint");
                }
                _ => {}
            }
        }
        rcs.gates.push(Gate {
            a: x.lc.0 * y.lc.2,
            x: x.lc.1,
            b: x.lc.2 * y.lc.0,
            y: y.lc.1,
            c: -z.lc.0,
            z: z.lc.1,
            d: x.lc.0 * y.lc.0,
            e: x.lc.2 * y.lc.2 - z.lc.2,
        })
    }

    fn enforce_add(x: &CNum<Self>, y: &CNum<Self>, z: &CNum<Self>) {
        let mut rcs = x.get_cs().borrow_mut();
        if rcs.tracking {
            match (x.value, y.value, z.value) {
                (Some(x), Some(y), Some(z)) => {
                    assert!(x + y == z, "Not satisfied constraint");
                }
                _ => {}
            }
        }
        rcs.gates.push(Gate {
            a: x.lc.0,
            x: x.lc.1,
            b: y.lc.0,
            y: y.lc.1,
            c: -z.lc.0,
            z: z.lc.1,
            d: Num::ZERO,
            e: x.lc.2 + y.lc.2 - z.lc.2,
        })
    }

    fn inputize(n: &CNum<Self>) {
        let v = if n.lc.0 == Num::ONE && n.lc.2 == Num::ZERO {
            n.lc.1
        } else {
            let m: CNum<Self> = n.derive_alloc(n.value.as_ref());
            m.assert_eq(n);
            m.lc.1
        };

        n.get_cs().borrow_mut().public.push(v);
    }

    fn alloc(cs: &RCS<Self>, value: Option<&Num<Fr>>) -> CNum<Self> {
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
