use bellman::pairing::{
    Engine,
};

use bellman::pairing::ff::{
    Field
};

use bellman::{
    SynthesisError,
    ConstraintSystem,
    LinearCombination,
    Variable,
    Index
};


use std::ops::{Add, Sub, Mul, Neg};
use std::collections::HashMap;

use super::Assignment;



#[derive(Clone)]
pub enum Signal<E:Engine> {
    Variable(Option<E::Fr>, LinearCombination<E>),
    Constant(E::Fr)
}

impl<E: Engine> Neg for Signal<E> {
    type Output = Signal<E>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Constant(mut a) => {a.negate(); Self::Constant(a)},
            Self::Variable(mut value, mut lc) => {
                value = value.map(|mut v| {v.negate(); v});
                lc = LinearCombination::zero() - &lc;
                Self::Variable(value, lc)
            }
        }
    }
}

impl<'a, E: Engine> Neg for &'a Signal<E> {
    type Output = Signal<E>;

    fn neg(self) -> Self::Output {
        match self {
            &Signal::Constant(mut a) => {a.negate(); Signal::Constant(a)},
            _ => {
                let mut value = self.get_value();
                value = value.map(|mut v| {v.negate(); v});
                let lc = LinearCombination::zero() - &self.lc();
                Signal::Variable(value, lc)
            }
        }
    }
}



impl<'a, E: Engine> Add<&'a Signal<E>> for Signal<E> {
    type Output = Signal<E>;

    fn add(self, other: &'a Signal<E>) -> Self::Output {
        match (&self, other) {
            (&Self::Constant(mut a), Self::Constant(b)) => {
                a.add_assign(b);
                Self::Constant(a)
            },
            _ => {
                let value = match (self.get_value(), other.get_value()) {
                    (Some(mut a), Some(b)) => {a.add_assign(&b); Some(a)},
                    _ => None
                };
                let lc = self.lc() + &other.lc();
                Self::Variable(value, lc)
            }
        }
    }
}


impl<'a, 'b, E: Engine> Add<&'a Signal<E>> for &'b Signal<E> {
    type Output = Signal<E>;

    fn add(self, other: &'a Signal<E>) -> Self::Output {
        match (self, other) {
            (&Signal::Constant(mut a), Signal::Constant(b)) => {
                a.add_assign(b);
                Signal::Constant(a)
            },
            _ => {
                let value = match (self.get_value(), other.get_value()) {
                    (Some(mut a), Some(b)) => {a.add_assign(&b); Some(a)},
                    _ => None
                };
                let lc = self.lc() + &other.lc();
                Signal::Variable(value, lc)
            }
        }
    }
}


impl<'a, E: Engine> Sub<&'a Signal<E>> for Signal<E> {
    type Output = Signal<E>;

    fn sub(self, other: &'a Signal<E>) -> Self::Output {
        match (&self, other) {
            (&Self::Constant(mut a), Self::Constant(b)) => {
                a.sub_assign(b);
                Self::Constant(a)
            },
            _ => {
                let value = match (self.get_value(), other.get_value()) {
                    (Some(mut a), Some(b)) => {a.sub_assign(&b); Some(a)},
                    _ => None
                };
                let lc = self.lc() - &other.lc();
                Self::Variable(value, lc)
            }
        }
    }
}


impl<'a, 'b, E: Engine> Sub<&'a Signal<E>> for &'b Signal<E> {
    type Output = Signal<E>;

    fn sub(self, other: &'a Signal<E>) -> Self::Output {
        match (self, other) {
            (&Signal::Constant(mut a), Signal::Constant(b)) => {
                a.sub_assign(b);
                Signal::Constant(a)
            },
            _ => {
                let value = match (self.get_value(), other.get_value()) {
                    (Some(mut a), Some(b)) => {a.sub_assign(&b); Some(a)},
                    _ => None
                };
                let lc = self.lc() - &other.lc();
                Signal::Variable(value, lc)
            }
        }
    }
}


impl<'a, E: Engine> Mul<&'a Signal<E>> for E::Fr {
    type Output = Signal<E>;

    fn mul(self, other: &'a Signal<E>) -> Self::Output {
        match other {
            &Signal::Constant(mut a) => {
                a.mul_assign(&self);
                Signal::Constant(a)
            },
            _ => {
                let value = match other.get_value() {
                    Some(mut a) => {a.mul_assign(&self); Some(a)},
                    _ => None
                };
                let lc = LinearCombination::<E>::zero() + (self, &other.lc());
                Signal::Variable(value, lc)
            }
        }
    }
}





impl <E:Engine> Signal<E> {
    pub fn get_value(&self) -> Option<E::Fr> {
        match self {
            &Self::Variable(v, _) => v,
            &Self::Constant(v) => Some(v)
        }
    }

    pub fn lc(&self) -> LinearCombination<E> {
        match self {
            Self::Variable(_, lc) => lc.clone(),
            Self::Constant(v) => LinearCombination::<E>::zero() + (*v, Variable::new_unchecked(Index::Input(0)))
        }
    }

    pub fn one() -> Self {
        Self::Constant(E::Fr::one())
    }

    pub fn zero() -> Self {
        Self::Constant(E::Fr::zero())
    }

    pub fn normalize(&self) -> Self {
        match self {
            Self::Variable(value, lc) => {
                let mut hm = HashMap::new();
                for (var, coeff) in lc.as_ref() {
                    hm.entry(var).or_insert(E::Fr::zero()).add_assign(coeff);
                }

                let mut lc = LinearCombination::<E>::zero();
                for (var, coeff) in hm {
                    if !coeff.is_zero() {
                        lc = lc + (coeff, *var);
                    }
                }

                let lc_items = lc.as_ref();


                if lc_items.len()==0 {
                    Self::Constant(E::Fr::zero())
                } else if lc_items.len()==1 && lc_items[0].0.get_unchecked() == Index::Input(0) {
                    Self::Constant(lc_items[0].1)
                } else {
                    Self::Variable(*value, lc)
                }
            },
            &Self::Constant(v) => Self::Constant(v)
        }
    }


    pub fn alloc<CS, F>(
        mut cs: CS,
        value: F,
    ) -> Result<Self, SynthesisError>
        where CS: ConstraintSystem<E>,
              F: FnOnce() -> Result<E::Fr, SynthesisError>
    {
        let mut new_value = None;
        let var = cs.alloc(|| "num", || {
            let tmp = value()?;
            new_value = Some(tmp);
            Ok(tmp)
        })?;

        Ok(Self::Variable(new_value, LinearCombination::<E>::zero() + (E::Fr::one(), var)))
    }

    pub fn inputize<CS>(
        &self,
        mut cs: CS
    ) -> Result<(), SynthesisError>
        where CS: ConstraintSystem<E>
    {
         match self {
            Self::Variable(v, lc) => {
                let input = cs.alloc_input(
                    || "input variable",
                    || v.grab()
                )?;

                cs.enforce(
                    || "enforce input is correct",
                    |zero| zero + input,
                    |zero| zero + CS::one(),
                    |zero| zero + lc
                );
                Ok(())
            },
            &Self::Constant(v) => {
                let input = cs.alloc_input(
                    || "input variable",
                    || Ok(v)
                )?;
        
                cs.enforce(
                    || "enforce input is correct",
                    |zero| zero + input,
                    |zero| zero + CS::one(),
                    |zero| zero + (v, CS::one())
                );
                Ok(())
            }
        }
    }

    pub fn multiply<CS:ConstraintSystem<E>>(&self, mut cs: CS, b: &Self) -> Result<Self, SynthesisError> {
        let a = self.normalize();
        let b = b.normalize();
        
        
        let a_mul_b_value = match (a.get_value(), b.get_value()) {
            (Some(mut a), Some(b)) => {a.mul_assign(&b); Some(a)},
            _ => None
        };

        let signal = match (a, b) {
            (Self::Constant(_), Self::Constant(_)) => Self::Constant(a_mul_b_value.unwrap()),
            (Self::Constant(a), b) => {
                if a.is_zero() {
                    Self::zero()
                } else {
                    Self::Variable(a_mul_b_value, LinearCombination::<E>::zero() + (a, &b.lc()))
                }
            },  
            (a, Self::Constant(b)) => {
                if b.is_zero() {
                    Self::zero()
                } else {
                    Self::Variable(a_mul_b_value, LinearCombination::<E>::zero() + (b, &a.lc()))
                }
            },
            (a, b) => {
                let a_mul_b = cs.alloc(|| "a mul b", || a_mul_b_value.grab())?;
                let a_mul_b_lc = LinearCombination::<E>::zero() + a_mul_b;
                cs.enforce(|| "enforce res == a mul b", |_| a.lc(), |_| b.lc(), |zero| zero + &a_mul_b_lc);
                Self::Variable(a_mul_b_value, a_mul_b_lc)
            }
        };
        Ok(signal)
    }

    pub fn divide<CS:ConstraintSystem<E>>(&self, mut cs: CS, b: &Self) -> Result<Self, SynthesisError> {
        let a = self.normalize();
        let b = b.normalize();

        let b_value = b.get_value();
        
        if let Some(t) = b_value {
            if t.is_zero() {
                return Err(SynthesisError::DivisionByZero);
            }
        }

        let b_inverse_value = b_value.map(|x| x.inverse().unwrap());
        
        
        let a_div_b_value = match (a.get_value(), b_inverse_value) {
            (Some(mut a), Some(b_inv)) => {a.mul_assign(&b_inv); Some(a)},
            _ => None
        };

        let signal = match (a, b) {
            (Self::Constant(_), Self::Constant(_)) => Self::Constant(a_div_b_value.unwrap()), 
            (a, Self::Constant(_)) => {
                Self::Variable(a_div_b_value, LinearCombination::<E>::zero() + (b_inverse_value.unwrap(), &a.lc()))
            },
            (a, b) => {
                let a_div_b = cs.alloc(|| "a mul b", || a_div_b_value.grab())?;
                let a_div_b_lc = LinearCombination::<E>::zero() + a_div_b;
                cs.enforce(|| "enforce res * b == a ", |zero| zero + &a_div_b_lc, |_| b.lc(), |_| a.lc());
                Self::Variable(a_div_b_value, a_div_b_lc)
            }
        };
        Ok(signal)
    }

    pub fn square<CS:ConstraintSystem<E>>(&self, mut cs: CS) -> Result<Self, SynthesisError> {
        self.multiply(cs.namespace(|| "multiply self*self"), self)
    }


    pub fn assert_zero<CS:ConstraintSystem<E>>(&self, mut cs:CS) -> Result<(), SynthesisError> {
        match self {
            Signal::Constant(c) => {
                if c.is_zero() {
                    Ok(())
                } else {
                    Err(SynthesisError::Unsatisfiable)
                }
            },
            Signal::Variable(_, _) => {
                cs.enforce(|| "0*0==self", |zero| zero, |zero| zero, |_| self.lc());
                Ok(())
            }
        }
    }

    pub fn assert_bit<CS:ConstraintSystem<E>>(&self, mut cs:CS) -> Result<(), SynthesisError> {
        match self {
            Signal::Constant(c) => {
                let mut r = c.clone();
                r.sub_assign(&E::Fr::one());
                r.mul_assign(&c);
                if r.is_zero() {
                    Ok(())
                } else {
                    Err(SynthesisError::Unsatisfiable)
                }
                
            },
            Signal::Variable(_, _) => {
                cs.enforce(|| "self*(self-1)==self", |_| self.lc(), |_| self.lc() - (E::Fr::one(), CS::one()), |zero| zero);
                Ok(())
            }
        }
        
    }

}