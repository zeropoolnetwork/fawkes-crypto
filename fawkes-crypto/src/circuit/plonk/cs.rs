use ff_uint::{Num, PrimeField};
use super::{
    super::general::{traits::signal::Signal, Variable},
    num::CNum
};



#[derive(Clone, Debug)]
pub enum Gate<Fr:PrimeField> {
    Pub(Variable),
    // a*x + b *y + c*z + d*x*y + e == 0
    Arith(Num<Fr>, Variable, Num<Fr>, Variable, Num<Fr>, Variable, Num<Fr>, Num<Fr>)
}
#[derive(Clone, Debug)]
pub struct CS<Fr:PrimeField> {
    pub n_vars: usize,
    pub gates: Vec<Gate<Fr>>,
    pub tracking:bool
}


impl<Fr:PrimeField> CS<Fr> {
    // a*b === c
    pub fn enforce_mul(a:&CNum<Fr>, b:&CNum<Fr>, c:&CNum<Fr>) {
        let mut rcs = a.get_cs().borrow_mut();
        if rcs.tracking {
            match (a.value, b.value, c.value) {
                (Some(a), Some(b), Some(c)) => {
                    assert!(a*b==c, "Not satisfied constraint");
                },
                _ => {}
            } 
        }
        rcs.gates.push(
            Gate::Arith(a.lc.0*b.lc.2, a.lc.1, a.lc.2*b.lc.0, b.lc.1, -c.lc.0, c.lc.1, a.lc.0*b.lc.0, a.lc.2*b.lc.2 - c.lc.2)
        )

    }

    pub fn enforce_add(a:&CNum<Fr>, b:&CNum<Fr>, c:&CNum<Fr>) {
        let mut rcs = a.get_cs().borrow_mut();
        if rcs.tracking {
            match (a.value, b.value, c.value) {
                (Some(a), Some(b), Some(c)) => {
                    assert!(a+b==c, "Not satisfied constraint");
                },
                _ => {}
            } 
        }
        rcs.gates.push(
            Gate::Arith(a.lc.0, a.lc.1, b.lc.0, b.lc.1, -c.lc.0, c.lc.1, Num::ZERO, a.lc.2 + b.lc.2 - c.lc.2)
        ) 
    }

    pub fn enforce_pub(n:&CNum<Fr>) {
        assert!(n.lc.0 == Num::ONE && n.lc.2 == Num::ZERO, "Wrong pub signal format");
        n.get_cs().borrow_mut().gates.push(Gate::Pub(n.lc.1));
    }
}