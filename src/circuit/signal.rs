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
use maplit::hashmap;

use super::Assignment;
use crate::wrappedmath::Wrap;



#[derive(Clone)]
pub enum Signal<E:Engine> {
    Variable(Option<Wrap<E::Fr>>, HashMap<Variable, Wrap<E::Fr>>),
    Constant(Wrap<E::Fr>)
}

pub fn enforce<E:Engine, CS:ConstraintSystem<E>>(mut cs:CS, a:&Signal<E>, b:&Signal<E>, c:&Signal<E>) {
    cs.enforce(|| "enforce", |_| a.lc(), |_| b.lc(), |_| c.lc());
}

fn _neg<E:Engine>(a:&Signal<E>) -> Signal<E> {
    match a {
        &Signal::Constant(a) => Signal::Constant(-a),
        Signal::Variable(value, m) => {
            let value = value.map(|x| -x);
            let mut m = m.clone();
            for (_, v) in m.iter_mut() {
                *v = -*v;
            }
            Signal::Variable(value, m)
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
            let mut a_m = a.get_varmap();
            let b_m = b.get_varmap();
            for (k, v) in b_m {
                if {
                    let t = a_m.entry(k).or_insert(Wrap::zero());
                    *t+=v;
                    t.is_zero()
                } {
                    a_m.remove(&k);
                }
                
            }

            Signal::Variable(value, a_m)
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
            let mut a_m = a.get_varmap();
            let b_m = b.get_varmap();
            for (k, v) in b_m {
                if {
                    let t = a_m.entry(k).or_insert(Wrap::zero());
                    *t-=v;
                    t.is_zero()
                } {
                    a_m.remove(&k);
                }
                
            }

            Signal::Variable(value, a_m)
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
    if a.is_zero() {
        Signal::Constant(Wrap::zero())
    } else {
        match b {
            &Signal::Constant(b) => Signal::Constant(a*b),
            _ => {
                let value = match b.get_value() {
                    Some(b) => Some(a*b),
                    _ => None
                };
                let mut b_m = b.get_varmap();
                for (_, v) in b_m.iter_mut() {
                    *v *= a;
                }
                Signal::Variable(value, b_m)
            }
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

    pub fn get_varmap(&self) -> HashMap<Variable, Wrap<E::Fr>> {
        match self {
            Self::Variable(_, m) => m.clone(),
            &Self::Constant(v) => hashmap!{Variable::new_unchecked(Index::Input(0)) => v}
        }
    }

    pub fn lc(&self) -> LinearCombination<E> {
        match self {
            Self::Variable(_, m) => {
                let mut acc = LinearCombination::<E>::zero();
                for (k, v) in m {
                    acc = acc + (v.into_inner(), *k)
                }
                acc
            }
            Self::Constant(v) => LinearCombination::<E>::zero() + (v.into_inner(), Variable::new_unchecked(Index::Input(0)))
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Variable(_, m) => m.keys().len(),
           _ => 1
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
            Self::Variable(_, m) => {
                let lc_items = m.iter().collect::<Vec<_>>();

                if lc_items.len()==0 {
                    Self::Constant(Wrap::zero())
                } else if lc_items.len()==1 && lc_items[0].0.get_unchecked() == Index::Input(0) {
                    Self::Constant(*lc_items[0].1)
                } else {
                    self.clone()
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
        Ok(Self::Variable(value, hashmap!{var =>Wrap::one()}))
    }

    pub fn inputize<CS>(
        &self,
        mut cs: CS
    ) -> Result<(), SynthesisError>
        where CS: ConstraintSystem<E>
    {
         match self {
            Self::Variable(v, _) => {
                let input = cs.alloc_input(
                    || "input variable",
                    || v.grab()
                )?;

                cs.enforce(
                    || "enforce input is correct",
                    |zero| zero + input,
                    |zero| zero + CS::one(),
                    |zero| zero + &self.lc()
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
    

        let signal = match (a, b) {
            (Self::Constant(a), Self::Constant(b)) => Self::Constant(a*b),
            (Self::Constant(a), b) => {
                if a.is_zero() {
                    Self::zero()
                } else {
                    a * &b
                }
            },  
            (a, Self::Constant(b)) => {
                if b.is_zero() {
                    Self::zero()
                } else {
                    b * &a
                }
            },
            (a, b) => {
                let a_mul_b_value = match (a.get_value(), b.get_value()) {
                    (Some(a), Some(b)) => Some(a*b),
                    _ => None
                };
                let a_mul_b = cs.alloc(|| "a mul b", || a_mul_b_value.grab())?;
                cs.enforce(|| "<== a mul b", |_| a.lc(), |_| b.lc(), |zero| zero + a_mul_b);
                Self::Variable(a_mul_b_value, hashmap!{a_mul_b => Wrap::one()})
            }
        };
        Ok(signal)
    }

    pub fn divide<CS:ConstraintSystem<E>>(&self, mut cs: CS, b: &Self) -> Result<Self, SynthesisError> {
        let a = self.normalize();
        let b = b.normalize();
        let signal = match (a, b) {
            (Self::Constant(a), Self::Constant(b)) => Self::Constant(a*b.inverse().ok_or(SynthesisError::DivisionByZero)?),
            (a, Self::Constant(b)) => b.inverse().ok_or(SynthesisError::DivisionByZero)? * &a,
            (a, b) => {

                let a_div_b_value = match (a.get_value(), b.get_value()) {
                    (Some(a), Some(b)) => Some(a*b.inverse().ok_or(SynthesisError::DivisionByZero)?),
                    _ => None
                };
                let a_div_b = cs.alloc(|| "a div b", || a_div_b_value.grab())?;
                cs.enforce(|| "(a div b) * b == a ", |zero| zero + a_div_b, |_| b.lc(), |_| a.lc());
                Self::Variable(a_div_b_value, hashmap!{a_div_b => Wrap::one()})
            }
        };
        Ok(signal)
    }

    pub fn square<CS:ConstraintSystem<E>>(&self, mut cs: CS) -> Result<Self, SynthesisError> {
        self.multiply(cs.namespace(|| "square"), self)
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
            _ => if if_else.len() < self.len() {
                Ok(if_else + &bit.multiply(cs.namespace(|| "compute flag*(if_ok-if_else)"), &(self-if_else))?)
            } else {
                Ok(self + &(Signal::one() - bit).multiply(cs.namespace(|| "compute flag*(if_ok-if_else)"), &(if_else - self))?)
            }
            
            
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
                cs.enforce(|| "self==0", |zero| zero, |zero| zero, |_| self.lc());
                Ok(())
            }
        }
    }

    pub fn assert_constant<CS:ConstraintSystem<E>>(&self, mut cs:CS, c: Wrap<E::Fr>) -> Result<(), SynthesisError> {
        match self {
            Signal::Constant(con) => {
                if *con == c {
                    Ok(())
                } else {
                    Err(SynthesisError::Unsatisfiable)
                }
            },
            Signal::Variable(_, _) => {
                cs.enforce(|| "self==const", |zero| zero+(c.into_inner(), CS::one()), |zero| zero+CS::one(), |_| self.lc());
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


#[cfg(test)]
mod signal_test {
    use super::*;
    use sapling_crypto::circuit::test::TestConstraintSystem;
    use bellman::pairing::bn256::{Bn256};
    use rand::{Rng, thread_rng};


    #[test]
    fn alloc() {
        let mut rng = thread_rng();
        let a = rng.gen();

        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_a = Signal::alloc(cs.namespace(||"a"), Some(a)).unwrap();

        assert!(cs.is_satisfied(), "cs should be satisfied");
        assert!(signal_a.get_value().unwrap() == a);

    }


    #[test]
    fn addition() {
        let mut rng = thread_rng();
        let a = rng.gen();
        let b = rng.gen();

        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_a = Signal::alloc(cs.namespace(||"a"), Some(a)).unwrap();
        let signal_b = Signal::alloc(cs.namespace(||"b"), Some(b)).unwrap();

        let signal_a_plus_b = &signal_a + &signal_b;

        assert!(cs.is_satisfied(), "cs should be satisfied");
        assert!(signal_a_plus_b.get_value().unwrap() == a+b);

    }


    #[test]
    fn multiply() {
        let mut rng = thread_rng();
        let a = rng.gen();
        let b = rng.gen();

        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_a = Signal::alloc(cs.namespace(||"a"), Some(a)).unwrap();
        let signal_b = Signal::alloc(cs.namespace(||"b"), Some(b)).unwrap();

        let signal_a_mul_b = signal_a.multiply(cs.namespace(|| "mul"), &signal_b).unwrap();

        assert!(cs.is_satisfied(), "cs should be satisfied");
        assert!(signal_a_mul_b.get_value().unwrap() == a*b);

    }


    #[test]
    fn normalize() {
        let mut rng = thread_rng();
        let a = rng.gen();
        let b = rng.gen();

        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_a = Signal::alloc(cs.namespace(||"a"), Some(a)).unwrap();
        let signal_b = Signal::alloc(cs.namespace(||"b"), Some(b)).unwrap();

        let signal_a_mul_3b = signal_a.multiply(cs.namespace(|| "mul"), &(&signal_b+&signal_b+&signal_b)).unwrap();

        assert!(cs.is_satisfied(), "cs should be satisfied");
        assert!(signal_a_mul_3b.get_value().unwrap() == a*b*Wrap::from(3u64));

    }
}