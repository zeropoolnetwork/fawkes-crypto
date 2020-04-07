use bellman::pairing::{
    Engine
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
use std::cmp::{Ordering};

use linked_list::{LinkedList, Cursor};

use super::Assignment;
use crate::wrappedmath::Wrap;

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct WrapVar(pub Index);

impl WrapVar {
    pub fn into_var(&self) -> Variable {
        Variable::new_unchecked(self.0)
    }

    fn one() -> Self {
        Self(Index::Input(0))
    }
}

impl PartialOrd for WrapVar {
    fn partial_cmp(&self, other: &WrapVar) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WrapVar {
    fn cmp(&self, other: &WrapVar) -> Ordering {
        match (self.0, other.0) {
            (Index::Input(a), Index::Input(b)) => a.cmp(&b),
            (Index::Input(_), Index::Aux(_)) => Ordering::Less,
            (Index::Aux(_), Index::Input(_)) => Ordering::Greater,
            (Index::Aux(a), Index::Aux(b)) => a.cmp(&b)
        }
    }
}


#[derive(Clone)]
pub enum Signal<E:Engine> {
    Variable(Option<Wrap<E::Fr>>, LinkedList<(WrapVar, Wrap<E::Fr>)>),
    Constant(Wrap<E::Fr>)
}


pub fn enforce<E:Engine, CS:ConstraintSystem<E>>(mut cs:CS, a:&Signal<E>, b:&Signal<E>, c:&Signal<E>) {
    cs.enforce(|| "enforce", |_| a.lc(), |_| b.lc(), |_| c.lc());
}

fn _neg<E:Engine>(a:&Signal<E>) -> Signal<E> {
    match a {
        &Signal::Constant(a) => Signal::Constant(-a),
        Signal::Variable(value, ll) => {
            let value = value.map(|x| -x);
            let mut ll = ll.clone();
            for (_, v) in ll.iter_mut() {
                *v = -*v;
            }
            Signal::Variable(value, ll)
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

#[derive(Eq,PartialEq)]
enum LookupAction {
    Add,
    Insert
}

fn ll_lookup<K:PartialEq+PartialOrd,V>(cur: &mut Cursor<(K, V)>, n: K) -> LookupAction {
    loop {
        match cur.peek_next() {
            Some((k, _)) => {
                if  *k == n {
                    return LookupAction::Add;
                } else if *k > n {
                    return  LookupAction::Insert;
                }
            },
            None => {
                return LookupAction::Insert;
            }
        }
        cur.seek_forward(1);
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
            let mut a_ll = a.get_varlist();
            let b_ll = b.get_varlist();
            let mut cur_a_ll = a_ll.cursor();

            for (k, v) in b_ll.iter() {
                if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                    let t = cur_a_ll.peek_next().unwrap();
                    t.1 += *v;
                    if t.1.is_zero() {
                        cur_a_ll.remove();
                    }
                } else {
                    cur_a_ll.insert((*k, *v))
                }
            }
            Signal::Variable(value, a_ll)
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
            let mut a_ll = a.get_varlist();
            let b_ll = b.get_varlist();
            let mut cur_a_ll = a_ll.cursor();

            for (k, v) in b_ll.iter() {
                if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                    let t = cur_a_ll.peek_next().unwrap();
                    t.1 -= *v;
                    if t.1.is_zero() {
                        cur_a_ll.remove();
                    }
                } else {
                    cur_a_ll.insert((*k, -*v))
                }
            }
            Signal::Variable(value, a_ll)
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
                let mut b_ll = b.get_varlist();
                for (_, v) in b_ll.iter_mut() {
                    *v *= a;
                }
                Signal::Variable(value, b_ll)
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

    pub fn get_varlist(&self) -> LinkedList<(WrapVar, Wrap<E::Fr>)> {
        match self {
            Self::Variable(_, ll) => ll.clone(),
            &Self::Constant(v) => {
                let mut ll = LinkedList::new();
                ll.push_back((WrapVar::one(), v));
                ll
            }
        }
    }

    pub fn from_var(value: Option<Wrap<E::Fr>>, var: Variable) -> Self{
        let mut ll = LinkedList::new();
        ll.push_back((WrapVar(var.get_unchecked()), Wrap::one()));
        Self::Variable(value, ll)
    }

    pub fn lc(&self) -> LinearCombination<E> {
        match self {
            Self::Variable(_, ll) => {
                // let mut acc = LinearCombination::<E>::zero();
                // for (k, v) in ll {
                //     acc = acc + (v.into_inner(), k.into_var())
                // }
                // acc
                let acc = ll.iter().map(|(k, v)| (k.into_var(), v.into_inner())).collect::<Vec<_>>();
                unsafe {std::mem::transmute(acc)}

            }
            Self::Constant(v) => LinearCombination::<E>::zero() + (v.into_inner(), WrapVar::one().into_var())
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Variable(_, ll) => ll.len(),
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
            Self::Variable(_, ll) => {
                if ll.len() > 1 {
                    self.clone()
                } else if ll.len() == 0 {
                    Self::Constant(Wrap::zero())
                } else {
                    let front = ll.front().unwrap();
                    if front.0 == WrapVar::one() {
                        Self::Constant(front.1)
                    } else {
                        self.clone()
                    }
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
        Ok(Self::from_var(value, var))
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
                Self::from_var(a_mul_b_value, a_mul_b)
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
                Self::from_var(a_div_b_value, a_div_b)
                
            }
        };
        Ok(signal)
    }

    pub fn square<CS:ConstraintSystem<E>>(&self, mut cs: CS) -> Result<Self, SynthesisError> {
        self.multiply(cs.namespace(|| "^2"), self)
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
                let inv_signal = Self::alloc(cs.namespace(|| ":=inv"), inv_value)?;
                let res_signal = self.multiply(cs.namespace(|| "s inv"), &inv_signal)?;

                inv_signal.assert_nonzero(cs.namespace(|| "inv_nonzero"))?;
                res_signal.assert_bit(cs.namespace(|| "res_bit"))?;
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
                Ok(if_else + &bit.multiply(cs.namespace(|| "switch"), &(self-if_else))?)
            } else {
                Ok(self + &(Signal::one() - bit).multiply(cs.namespace(|| "switch"), &(if_else - self))?)
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
                let inv_signal = Self::alloc(cs.namespace(|| ":=inv"), inv_value)?;
                cs.enforce(|| "s*invl==1", |_| self.lc(), |_| inv_signal.lc(), |zero| zero + CS::one());
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
                cs.enforce(|| "assert_bit", |_| self.lc(), |_| self.lc() - (E::Fr::one(), CS::one()), |zero| zero);
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