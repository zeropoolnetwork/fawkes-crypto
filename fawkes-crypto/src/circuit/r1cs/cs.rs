use crate::{
    circuit::num::CNum,
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
};
use linked_list::LinkedList;
use std::{cell::RefCell, rc::Rc};

pub type RCS<Fr: PrimeField> = Rc<RefCell<dyn CS<Fr = Fr>>>;

#[derive(Clone, Debug)]
pub struct LC<Fr: PrimeField>(pub LinkedList<(Num<Fr>, usize)>);

impl<Fr: PrimeField> LC<Fr> {
    pub fn to_vec(&self) -> Vec<(Num<Fr>, usize)> {
        self.0.iter().cloned().collect()
    }
}

#[derive(Clone, Debug)]
pub struct Gate<Fr: PrimeField>(
    pub Vec<(Num<Fr>, usize)>,
    pub Vec<(Num<Fr>, usize)>,
    pub Vec<(Num<Fr>, usize)>,
);

pub trait CS {
    type Fr: PrimeField;

    fn num_constraints(&self) -> usize;
    // a*b === c
    fn enforce(&self, a: &CNum<Self::Fr>, b: &CNum<Self::Fr>, c: &CNum<Self::Fr>);
    fn enforce_pub(&self, n: &CNum<Self::Fr>);
    fn alloc(&self, cs: &RCS<Self::Fr>, value: Option<&Num<Self::Fr>>) -> CNum<Self::Fr>;
}

#[derive(Clone, Debug)]
pub struct SetupCS<Fr: PrimeField> {
    pub values: Vec<Option<Num<Fr>>>,
    pub gates: Vec<Gate<Fr>>,
    pub tracking: bool,
    pub public: Vec<usize>,
}

impl<Fr: PrimeField> SetupCS<Fr> {
    fn new(tracking: bool) -> Self {
        Self {
            values: vec![Some(Num::ONE)],
            gates: vec![],
            tracking,
            public: vec![0],
        }
    }

    fn rc_new(tracking: bool) -> RCS<Fr> {
        Rc::new(RefCell::new(Self::new(tracking)))
    }
}

impl<Fr: PrimeField> CS for SetupCS<Fr> {
    type Fr = Fr;

    fn num_constraints(&self) -> usize {
        self.gates.len()
    }

    // a*b === c
    fn enforce(&self, a: &CNum<Fr>, b: &CNum<Fr>, c: &CNum<Fr>) {
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

    fn enforce_pub(&self, n: &CNum<Fr>) {
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

    fn alloc(&self, cs: &RCS<Fr>, value: Option<&Num<Fr>>) -> CNum<Fr> {
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
