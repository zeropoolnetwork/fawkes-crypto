use ff_uint::{Num, PrimeField};
use crate::circuit::{
    Variable,
    num::CNum
};
use crate::core::signal::Signal;

use std::cell::RefCell;
use std::rc::Rc;
pub type RCS<Fr> = Rc<RefCell<CS<Fr>>>;

use linked_list::LinkedList;

#[derive(Clone, Debug)]
pub struct LC<Fr:PrimeField>(pub LinkedList<(Num<Fr>, Variable)>);

impl<Fr:PrimeField> LC<Fr> {
    pub fn to_vec(&self) -> Vec<(Num<Fr>, Variable)> {
        self.0.iter().cloned().collect()
    }
}


#[derive(Clone, Debug)]
pub struct Gate<Fr:PrimeField>(pub Vec<(Num<Fr>, Variable)>,pub Vec<(Num<Fr>, Variable)>,pub Vec<(Num<Fr>, Variable)>);


#[derive(Clone, Debug)]
pub struct CS<Fr:PrimeField> {
    pub values: Vec<Option<Num<Fr>>>,
    pub gates: Vec<Gate<Fr>>,
    pub tracking:bool,
    pub public:Vec<Variable>
}


impl<Fr:PrimeField> CS<Fr> {
    pub fn num_constraints(&self) -> usize {
       self.gates.len() 
    }

    pub fn new(tracking:bool) -> Self {
        Self {
            values:vec![Some(Num::ONE)],
            gates:vec![],
            tracking,
            public:vec![0]
        }
    }

    pub fn rc_new(tracking:bool) -> RCS<Fr> {
        Rc::new(RefCell::new(Self::new(tracking)))
    }

    // a*b === c
    pub fn enforce(a:&CNum<Fr>, b:&CNum<Fr>, c:&CNum<Fr>) {
        let mut rcs = a.get_cs().borrow_mut();
        if rcs.tracking {
            match (a.value, b.value, c.value) {
                (Some(a), Some(b), Some(c)) => {
                    assert!(a*b==c, "Not satisfied constraint");
                },
                _ => {}
            } 
        }
        rcs.gates.push(Gate(a.lc.to_vec(), b.lc.to_vec(), c.lc.to_vec()))

    }


    pub fn enforce_pub(n:&CNum<Fr>) {
        let mut rcs = n.get_cs().borrow_mut();
        let n_vars = rcs.values.len();
        let v = n_vars;
        rcs.values.push(n.get_value());
        rcs.public.push(v);
        rcs.gates.push(Gate(n.lc.to_vec(), vec![(Num::ONE, 0)], vec![(Num::ONE, v)]));
    }


    pub fn alloc(cs:&RCS<Fr>, value:Option<&Num<Fr>>) -> CNum<Fr> {
        let mut rcs = cs.borrow_mut();
        let n_vars = rcs.values.len();
        let v = n_vars;
        rcs.values.push(value.cloned());
        let mut ll = LinkedList::new();
        ll.push_back((Num::ONE, v));
        CNum {value:value.cloned(), lc:LC(ll), cs:cs.clone()}
    }
}
