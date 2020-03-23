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
use crate::wrappedmath::Wrap;



#[derive(Clone)]
pub enum Signal<E:Engine> {
    Variable(Option<Wrap<E::Fr>>, LinearCombination<E>),
    Constant(Wrap<E::Fr>)
}

pub fn enforce<E:Engine, CS:ConstraintSystem<E>>(mut cs:CS, a:&Signal<E>, b:&Signal<E>, c:&Signal<E>) {
    cs.enforce(|| "enforce", |_| a.lc(), |_| b.lc(), |_| c.lc());
}

fn _neg<E:Engine>(a:&Signal<E>) -> Signal<E> {
    match a {
        &Signal::Constant(a) => Signal::Constant(-a),
        _ => {
            let value = a.get_value().map(|x| -x);
            let lc = LinearCombination::zero() - &a.lc();
            Signal::Variable(value, lc)
        }
    }
}


impl<E: Engine> Neg for Signal<E> {
    type Output = Signal<E>;
    fn neg(self) -> Self::Output {
        _neg(&self)
    }
}


impl<'a, E: Engine> Neg for &'a Signal<E> {
    type Output = Signal<E>;
    fn neg(self) -> Self::Output {
        _neg(self)
    }
}



fn _add<E: Engine>(a: &Signal<E>, b: &Signal<E>) -> Signal<E> {
    match (a, b) {
        (&Signal::Constant(a), &Signal::Constant(b)) => Signal::Constant(a+b),
        _ => {
            let value = match (a.get_value(), b.get_value()) {
                (Some(a), Some(b)) => {Some(a+b)},
                _ => None
            };
            let lc = a.lc() + &b.lc();
            Signal::Variable(value, lc)
        }
    }
}


impl<'a, E: Engine> Add<&'a Signal<E>> for Signal<E> {
    type Output = Signal<E>;
    fn add(self, other: &'a Signal<E>) -> Self::Output {
        _add(&self, other)
    }
}

impl<'a, 'b, E: Engine> Add<&'a Signal<E>> for &'b Signal<E> {
    type Output = Signal<E>;
    fn add(self, other: &'a Signal<E>) -> Self::Output {
        _add(self, other)
    }
}

impl<E: Engine> Add<Signal<E>> for Signal<E> {
    type Output = Signal<E>;
    fn add(self, other: Signal<E>) -> Self::Output {
        _add(&self, &other)
    }
}

impl<'b, E: Engine> Add<Signal<E>> for &'b Signal<E> {
    type Output = Signal<E>;
    fn add(self, other: Signal<E>) -> Self::Output {
        _add(self, &other)
    }
}


fn _sub<E: Engine>(a: &Signal<E>, b: &Signal<E>) -> Signal<E> {
    match (a, b) {
        (&Signal::Constant(a), &Signal::Constant(b)) => Signal::Constant(a-b),
        _ => {
            let value = match (a.get_value(), b.get_value()) {
                (Some(a), Some(b)) => {Some(a-b)},
                _ => None
            };
            let lc = a.lc() - &b.lc();
            Signal::Variable(value, lc)
        }
    }
}

impl<'a, E: Engine> Sub<&'a Signal<E>> for Signal<E> {
    type Output = Signal<E>;
    fn sub(self, other: &'a Signal<E>) -> Self::Output {
        _sub(&self, other)
    }
}

impl<'a, 'b, E: Engine> Sub<&'a Signal<E>> for &'b Signal<E> {
    type Output = Signal<E>;
    fn sub(self, other: &'a Signal<E>) -> Self::Output {
        _sub(self, other)
    }
}

impl<E: Engine> Sub<Signal<E>> for Signal<E> {
    type Output = Signal<E>;
    fn sub(self, other: Signal<E>) -> Self::Output {
        _sub(&self, &other)
    }
}

impl<'b, E: Engine> Sub<Signal<E>> for &'b Signal<E> {
    type Output = Signal<E>;
    fn sub(self, other: Signal<E>) -> Self::Output {
        _sub(self, &other)
    }
}


fn _mul<E:Engine>(a:Wrap<E::Fr>, b:&Signal<E>) -> Signal<E> {
    match b {
        &Signal::Constant(b) => Signal::Constant(a*b),
        _ => {
            let value = match b.get_value() {
                Some(b) => Some(a*b),
                _ => None
            };
            let lc = LinearCombination::<E>::zero() + (a.into_inner(), &b.lc());
            Signal::Variable(value, lc)
        }
    }
}


impl<'a, E: Engine> Mul<&'a Signal<E>> for Wrap<E::Fr> {
    type Output = Signal<E>;
    fn mul(self, other: &'a Signal<E>) -> Self::Output {
        _mul(self, other)
    }
}

impl<E: Engine> Mul<Signal<E>> for Wrap<E::Fr> {
    type Output = Signal<E>;
    fn mul(self, other: Signal<E>) -> Self::Output {
        _mul(self, &other)
    }
}


impl<'b, E: Engine> Mul<Wrap<E::Fr>> for &'b Signal<E>  {
    type Output = Signal<E>;
    fn mul(self, other: Wrap<E::Fr>) -> Self::Output {
        _mul(other, self)
    }
}

impl<E: Engine> Mul<Wrap<E::Fr>> for Signal<E> {
    type Output = Signal<E>;
    fn mul(self, other: Wrap<E::Fr>) -> Self::Output {
        _mul(other, &self)
    }
}





impl <E:Engine> Signal<E> {
    pub fn get_value(&self) -> Option<Wrap<E::Fr>> {
        match self {
            &Self::Variable(v, _) => v,
            &Self::Constant(v) => Some(v)
        }
    }

    pub fn lc(&self) -> LinearCombination<E> {
        match self {
            Self::Variable(_, lc) => lc.clone(),
            Self::Constant(v) => LinearCombination::<E>::zero() + (v.into_inner(), Variable::new_unchecked(Index::Input(0)))
        }
    }

    pub fn one() -> Self {
        Self::Constant(Wrap::one())
    }

    pub fn zero() -> Self {
        Self::Constant(Wrap::zero())
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
                    Self::Constant(Wrap::zero())
                } else if lc_items.len()==1 && lc_items[0].0.get_unchecked() == Index::Input(0) {
                    Self::Constant(Wrap::new(lc_items[0].1))
                } else {
                    Self::Variable(*value, lc)
                }
            },
            &Self::Constant(v) => Self::Constant(v)
        }
    }


    pub fn alloc<CS: ConstraintSystem<E>>(
        mut cs: CS,
        value: Option<Wrap<E::Fr>>,
    ) -> Result<Self, SynthesisError>
    {
        let var = cs.alloc(|| "num", || value.grab())?;
        Ok(Self::Variable(value, LinearCombination::<E>::zero() + (E::Fr::one(), var)))
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
                    || Ok(v.into_inner())
                )?;
        
                cs.enforce(
                    || "enforce input is correct",
                    |zero| zero + input,
                    |zero| zero + CS::one(),
                    |zero| zero + (v.into_inner(), CS::one())
                );
                Ok(())
            }
        }
    }

    pub fn multiply<CS:ConstraintSystem<E>>(&self, mut cs: CS, b: &Self) -> Result<Self, SynthesisError> {
        let a = self.normalize();
        let b = b.normalize();
        
        
        let a_mul_b_value = match (a.get_value(), b.get_value()) {
            (Some(a), Some(b)) => Some(a*b),
            _ => None
        };

        let signal = match (a, b) {
            (Self::Constant(_), Self::Constant(_)) => Self::Constant(a_mul_b_value.unwrap()),
            (Self::Constant(a), b) => {
                if a.is_zero() {
                    Self::zero()
                } else {
                    Self::Variable(a_mul_b_value, LinearCombination::<E>::zero() + (a.into_inner(), &b.lc()))
                }
            },  
            (a, Self::Constant(b)) => {
                if b.is_zero() {
                    Self::zero()
                } else {
                    Self::Variable(a_mul_b_value, LinearCombination::<E>::zero() + (b.into_inner(), &a.lc()))
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
            (Some(a), Some(b_inv)) => Some(a*b_inv),
            _ => None
        };

        let signal = match (a, b) {
            (Self::Constant(_), Self::Constant(_)) => Self::Constant(a_div_b_value.unwrap()), 
            (a, Self::Constant(_)) => {
                Self::Variable(a_div_b_value, LinearCombination::<E>::zero() + (b_inverse_value.unwrap().into_inner(), &a.lc()))
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

    pub fn is_zero<CS:ConstraintSystem<E>>(&self, mut cs:CS) -> Result<Self, SynthesisError> {
        match self {
            Signal::Constant(c) => {
                if c.is_zero() {
                    Ok(Signal::one())
                } else {
                    Ok(Signal::zero())
                }
            },
            Signal::Variable(value, _) => {
                let inv_value = match value {
                    Some(t) => t.inverse().or(Some(Wrap::one())),
                    None => None
                };
                let inv_signal = Self::alloc(cs.namespace(|| "alloc inverse value"), inv_value)?;
                let res_signal = self.multiply(cs.namespace(|| "compute signal*inv_signal"), &inv_signal)?;

                inv_signal.assert_nonzero(cs.namespace(|| "assert inv_signal nonzero"))?;
                res_signal.assert_bit(cs.namespace(|| "assert res_signal bit"))?;
                Ok(Signal::one() - &res_signal)
            }
        }
    }

    pub fn switch<CS:ConstraintSystem<E>>(&self, mut cs:CS, bit: &Self, if_else: &Self) -> Result<Self, SynthesisError> {
        match bit {
            &Signal::Constant(b) => {
                if b == Wrap::one() {
                    Ok(self.clone())
                } else {
                    if b.is_zero() {
                        Ok(if_else.clone())
                    }
                    else {
                        Err(SynthesisError::Unsatisfiable)
                    }
                }
            },
            Signal::Variable(_, _) => Ok(if_else + &bit.multiply(cs.namespace(|| "compute flag*(if_ok-if_else)"), &(self-if_else))?)
        }
        
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

    pub fn assert_nonzero<CS:ConstraintSystem<E>>(&self, mut cs:CS) -> Result<(), SynthesisError> {
        match self {
            Signal::Constant(c) => {
                if c.is_zero() {
                    Err(SynthesisError::Unsatisfiable)
                } else {
                    Ok(())
                }
            },
            Signal::Variable(value, _) => {
                let inv_value = match value {
                    Some(t) => Some(t.inverse().ok_or(SynthesisError::DivisionByZero)?),
                    None => None
                };
                let inv_signal = Self::alloc(cs.namespace(|| "alloc inverse value"), inv_value)?;
                cs.enforce(|| "signal*inv_signal==1", |_| self.lc(), |_| inv_signal.lc(), |zero| zero + CS::one());
                Ok(())
            }
        }
    }

    pub fn assert_bit<CS:ConstraintSystem<E>>(&self, mut cs:CS) -> Result<(), SynthesisError> {
        match self {
            &Signal::Constant(c) => {
                if c.is_zero() || c == Wrap::one() {
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