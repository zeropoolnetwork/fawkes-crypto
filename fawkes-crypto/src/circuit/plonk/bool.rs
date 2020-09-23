use ff_uint::{Num, PrimeField};
use crate::circuit::plonk::{num::CNum, cs::CS};
use crate::circuit::general::{traits::{signal::Signal, bool::SignalBool}, Variable};
use std::cell::RefCell;
use std::rc::Rc;


#[derive(Clone, Debug)]
pub struct CBool<Fr:PrimeField>(CNum<Fr>);

impl<Fr:PrimeField> SignalBool for CBool<Fr> {
    type Num=CNum<Fr>;
}

impl<Fr:PrimeField> Signal for CBool<Fr> {
    type Value = bool;
    type CS = Rc<RefCell<CS<Fr>>>;

    fn as_const(&self) -> Option<Self::Value> {
        let lc = self.0.lc;
        if lc.0 == Num::ZERO {
            if lc.2==Num::ZERO {
                Some(false)
            } else if lc.2==Num::ONE {
                Some(true)
            }   else {
                panic!("Wrong boolean value")
            }
        } else {
            None
        }
    }

    fn inputize(&self) {
        self.0.inputize()
    }

    fn get_value(&self) -> Option<Self::Value> {
        self.0.value.map(|v| {
            if v==Num::ZERO {
                false
            } else if v==Num::ONE {
                true
            }   else {
                panic!("Wrong boolean value")
            }
        })
    }

    fn from_const(cs:&Self::CS, value: &Self::Value) -> Self {
        Self::new_unchecked(&CNum::from_const(cs, &(*value).into()))
    }

    fn get_cs(&self) -> &Self::CS {
        &self.0.cs
    }

    fn alloc(cs:&Self::CS, value:Option<&Self::Value>) -> Self {
        let mut rcs = cs.borrow_mut();
        let value = value.map(|&b| Into::<Num<Fr>>::into(b));
        let v = Variable(rcs.n_vars);
        rcs.n_vars+=1;
        Self::new_unchecked(&CNum {value:value, lc:(Num::ONE, v, Num::ZERO), cs:cs.clone()})
    }

    fn assert_const(&self, value: &Self::Value) {
        CS::enforce_add(&self.to_num(), &self.derive_const(&Num::ZERO), &self.derive_const(&(*value).into()))
    }

}

impl<Fr:PrimeField> CBool<Fr> {
    pub fn new_unchecked(n:&CNum<Fr>) -> Self {
        CBool(n.clone())
    }

    pub fn new(n: &CNum<Fr>) -> Self {
        n.assert_bit();
        Self::new_unchecked(n)
    }

    pub fn to_num(&self) -> CNum<Fr> {
        self.0.clone()
    }
}
