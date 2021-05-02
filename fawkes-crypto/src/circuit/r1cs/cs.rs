use crate::{
    circuit::num::CNum,
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
};
use linked_list::LinkedList;
use std::{cell::RefCell, rc::Rc};

pub type RCS<C: CS> = Rc<RefCell<C>>;

#[derive(Clone, Debug)]
pub struct LC<C: CS>(pub LinkedList<(Num<C::Fr>, usize)>);

impl<C: CS> LC<C> {
    pub fn to_vec(&self) -> Vec<(Num<C::Fr>, usize)> {
        self.0.iter().cloned().collect()
    }
}

#[derive(Clone, Debug)]
pub struct Gate<C: CS>(
    pub Vec<(Num<C::Fr>, usize)>,
    pub Vec<(Num<C::Fr>, usize)>,
    pub Vec<(Num<C::Fr>, usize)>,
);

pub trait CS: Clone {
    type Fr: PrimeField;

    fn num_constraints(&self) -> usize;
    // a*b === c
    fn enforce(a: &CNum<Self>, b: &CNum<Self>, c: &CNum<Self>);
    fn enforce_pub(n: &CNum<Self>);
    fn alloc(cs: &RCS<Self>, value: Option<&Num<Self::Fr>>) -> CNum<Self>;
}

#[derive(Clone, Debug)]
pub struct SetupCS<Fr: PrimeField> {
    pub values: Vec<Option<Num<Fr>>>,
    pub gates: Vec<Gate<Self>>,
    pub tracking: bool,
    pub public: Vec<usize>,
}

impl<Fr: PrimeField> SetupCS<Fr> {
    pub fn new(tracking: bool) -> Self {
        Self {
            values: vec![Some(Num::ONE)],
            gates: vec![],
            tracking,
            public: vec![0],
        }
    }

    pub fn rc_new(tracking: bool) -> RCS<Self> {
        Rc::new(RefCell::new(Self::new(tracking)))
    }
}

impl<Fr: PrimeField> CS for SetupCS<Fr> {
    type Fr = Fr;

    fn num_constraints(&self) -> usize {
        self.gates.len()
    }

    // a*b === c
    fn enforce(a: &CNum<Self>, b: &CNum<Self>, c: &CNum<Self>) {
        let mut rcs = a.get_cs().borrow_mut();
        if rcs.tracking {
            match (a.value, b.value, c.value) {
                (Some(a), Some(b), Some(c)) => {
                    assert!(a * b == c, "Not satisfied constraint");
                }
                _ => {}
            }
        }
        rcs.gates
            .push(Gate(a.lc.to_vec(), b.lc.to_vec(), c.lc.to_vec()))
    }

    fn enforce_pub(n: &CNum<Self>) {
        let mut rcs = n.get_cs().borrow_mut();
        let n_vars = rcs.values.len();
        let v = n_vars;
        rcs.values.push(n.get_value());
        rcs.public.push(v);
        rcs.gates.push(Gate(
            n.lc.to_vec(),
            vec![(Num::ONE, 0)],
            vec![(Num::ONE, v)],
        ));
    }

    fn alloc(cs: &RCS<Self>, value: Option<&Num<Self::Fr>>) -> CNum<Self> {
        let mut rcs = cs.borrow_mut();
        let n_vars = rcs.values.len();
        let v = n_vars;
        rcs.values.push(value.cloned());
        let mut ll = LinkedList::new();
        ll.push_back((Num::ONE, v));
        CNum {
            value: value.cloned(),
            lc: LC(ll),
            cs: cs.clone(),
        }
    }
}
