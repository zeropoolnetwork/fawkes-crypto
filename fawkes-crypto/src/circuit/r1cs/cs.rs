use ff_uint::{Num, PrimeField};
use super::{
    super::general::{traits::signal::{Signal, RCS}, Variable},
    num::CNum
};

use linked_list::LinkedList;

#[derive(Clone, Debug)]
pub struct LC<Fr:PrimeField>(pub Num<Fr>, pub LinkedList<(Num<Fr>, Variable)>);


#[derive(Clone, Debug)]
pub struct Gate<Fr:PrimeField>(pub LC<Fr>,pub LC<Fr>,pub LC<Fr>);


#[derive(Clone, Debug)]
pub struct CS<Fr:PrimeField> {
    values: Vec<Option<Num<Fr>>>,
    gates: Vec<Gate<Fr>>,
    tracking:bool,
    public:Vec<Variable>
}


impl<Fr:PrimeField> CS<Fr> {
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
        rcs.gates.push(Gate(a.lc.clone(), b.lc.clone(), c.lc.clone()))

    }


    pub fn enforce_pub(n:&CNum<Fr>) {
        let f = n.lc.1.front();
        let v = if n.lc.0.is_zero() && n.lc.1.len()==1 && f.map(|v| v.0 == Num::ONE).unwrap_or(false) {
            f.unwrap().1
        } else {
            let m: CNum<Fr> = n.derive_alloc(n.value.as_ref());
            CS::enforce(n, &n.derive_const(&Num::ONE), &m);
            m.lc.1.front().unwrap().1
        };

        n.get_cs().borrow_mut().public.push(v);
    }

    pub fn alloc(cs:&RCS<Fr>, value:Option<&Num<Fr>>) -> CNum<Fr> {
        let mut rcs = cs.borrow_mut();
        let n_vars = rcs.values.len();
        let v = n_vars;
        rcs.values.push(value.cloned());
        let mut ll = LinkedList::new();
        ll.push_back((Num::ONE, v));
        CNum {value:value.cloned(), lc:LC(Num::ZERO, ll), cs:cs.clone()}
    }
}
