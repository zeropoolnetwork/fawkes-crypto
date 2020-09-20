use ff_uint::{Num, PrimeField};
use crate::circuit::general::Variable;
use crate::circuit::plonk::cs::{CS, Gate};
use crate::circuit::general::traits::signal::Signal;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct CNum<Fr:PrimeField> {
    pub value:Option<Num<Fr>>,
    // a*x + b
    pub lc: (Num<Fr>, Variable, Num<Fr>),
    pub cs: Rc<RefCell<CS<Fr>>>
}

impl<Fr:PrimeField> Signal for CNum<Fr> {
    type Value = Num<Fr>;

    fn as_const(&self) -> Option<Self::Value> {
        let lc = self.lc;
        if lc.0 == Num::ZERO {
            Some(lc.2)
        } else {
            None
        }
    }

    fn get_value(&self) -> Option<Self::Value> {
        self.value
    }
}

fn lc_mul<Fr:PrimeField>(lc:(Num<Fr>, Variable, Num<Fr>), m:Num<Fr>) -> (Num<Fr>, Variable, Num<Fr>) {
    (lc.0*m, lc.1, lc.2*m)
}


fn lc_add<Fr:PrimeField>(lc:(Num<Fr>, Variable, Num<Fr>), m:Num<Fr>) -> (Num<Fr>, Variable, Num<Fr>) {
    (lc.0, lc.1, lc.2+m)
}

impl<Fr:PrimeField> CNum<Fr> {
    
    pub fn wrapping_mul(&self, other:&Self) -> Self {
        let cs = self.cs.clone();
        if let Some(c) = self.as_const() {
            Self {
                value: other.value.map(|v| v*c),
                lc: lc_mul(other.lc, c),
                cs
            }
        } else if let Some(c) = other.as_const() {
            Self {
                value: self.value.map(|v| v*c),
                lc: lc_mul(self.lc, c),
                cs
            }
        } else {
            let var = Variable(cs.borrow().n_vars);
            let gate = Gate::Arith(other.lc.2*self.lc.0, self.lc.1, other.lc.0*self.lc.2, other.lc.1, Num::ONE, var, self.lc.0*other.lc.0, self.lc.2*other.lc.2);
            {
                let mut cs = cs.borrow_mut();
                cs.gates.push(gate);
                cs.n_vars+=1;
            }
            let value = self.value.map(|a| other.value.map(|b| a*b)).flatten();

            Self {
                value,
                lc: (-Num::ONE, var, Num::ZERO),
                cs
            }
        }
        
    }

    pub fn wrapping_add(&self, other:&Self) -> Self {
        let cs = self.cs.clone();
        if let Some(c) = self.as_const() {
            Self {
                value: other.value.map(|v| v+c),
                lc: lc_add(other.lc, c),
                cs
            }
        } else if let Some(c) = other.as_const() {
            Self {
                value: self.value.map(|v| v+c),
                lc: lc_add(self.lc, c),
                cs
            }
        } else if self.lc.1 == other.lc.1 {
            Self {
                value: self.value.map(|a| other.value.map(|b| a+b)).flatten(),
                lc: (self.lc.0+other.lc.0, self.lc.1, self.lc.2+other.lc.2),
                cs
            }
        } else {
            let var = Variable(cs.borrow().n_vars);
            let gate = Gate::Arith(self.lc.0, self.lc.1, other.lc.0, other.lc.1, Num::ONE, var, Num::ZERO, self.lc.2+other.lc.2);
            {
                let mut cs = cs.borrow_mut();
                cs.gates.push(gate);
                cs.n_vars+=1;
            }
            let value = self.value.map(|a| other.value.map(|b| a+b)).flatten();

            Self {
                value,
                lc: (-Num::ONE, var, Num::ZERO),
                cs
            }
        }
    }

    pub fn wrapping_sub(&self, other:&Self) -> Self {
        let cs = self.cs.clone();
        if let Some(c) = self.as_const() {
            Self {
                value: other.value.map(|v| c-v),
                lc: (-other.lc.0, other.lc.1, c-other.lc.2),
                cs
            }
        } else if let Some(c) = other.as_const() {
            Self {
                value: self.value.map(|v| v-c),
                lc: lc_add(self.lc, -c),
                cs
            }
        } else if self.lc.1 == other.lc.1 {
            Self {
                value: self.value.map(|a| other.value.map(|b| a-b)).flatten(),
                lc: (self.lc.0+other.lc.0, self.lc.1, self.lc.2+other.lc.2),
                cs
            }
        } else {
            let var = Variable(cs.borrow().n_vars);
            let gate = Gate::Arith(self.lc.0, self.lc.1, -other.lc.0, other.lc.1, Num::ONE, var, Num::ZERO, self.lc.2-other.lc.2);
            {
                let mut cs = cs.borrow_mut();
                cs.gates.push(gate);
                cs.n_vars+=1;
            }
            let value = self.value.map(|a| other.value.map(|b| a-b)).flatten();

            Self {
                value,
                lc: (-Num::ONE, var, Num::ZERO),
                cs
            }
        }
    }

    

}