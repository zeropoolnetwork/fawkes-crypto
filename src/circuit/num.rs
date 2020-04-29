use linked_list::{LinkedList, Cursor};
use std::cmp::{Ordering};
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign, SubAssign, MulAssign, DivAssign};

use crate::circuit::bool::CBool;
use crate::core::cs::ConstraintSystem;
use crate::core::signal::Signal;
use crate::native::num::Num;


#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum Index{
    Input(usize),
    Aux(usize)
}


impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Index) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Index {
    fn cmp(&self, other: &Index) -> Ordering {
        match (self, other) {
            (Index::Input(a), Index::Input(b)) => a.cmp(&b),
            (Index::Input(_), Index::Aux(_)) => Ordering::Less,
            (Index::Aux(_), Index::Input(_)) => Ordering::Greater,
            (Index::Aux(a), Index::Aux(b)) => a.cmp(&b)
        }
    }
}


#[derive(Clone, Debug)]
pub struct CNum<'a, CS:ConstraintSystem>{
    pub value:Option<Num<CS::F>>,
    pub lc:LinkedList<(Index,Num<CS::F>)>,
    pub cs:&'a CS
}




impl<'a, CS:ConstraintSystem> Signal<'a, CS> for CNum<'a, CS> {
    type Value = Num<CS::F>;

    #[inline]
    fn get_value(&self) -> Option<Self::Value> {
        self.value
    }

    fn switch(&self, bit: &CBool<'a, CS>, if_else: &Self) -> Self {
        match bit.as_const() {
            Some(b) => {
                if b {
                    self.clone()
                } else {
                    if_else.clone()
                } 
   
            },
            _ => if if_else.capacity() < self.capacity() {
                if_else + &bit.0 * (self - if_else)
            } else {
                self + (Num::one() - &bit.0) * (if_else - self)
            }
        }
    }

    fn inputize(&self) {
        match self.as_const() {
            Some(v) => {
                let input = self.cs.alloc_input(Some(v));
                self.cs.enforce(
                    &self.derive_var(Some(v), input), 
                    &self.derive_one(), 
                    &self.derive_const(&v));

            },
            _ => {
                let input = self.cs.alloc_input(self.get_value());
                self.cs.enforce(
                    &self.derive_var(self.get_value(), input), 
                    &self.derive_one(), 
                    self);
            },
        }
    }

    fn linearize_builder(&self, acc: &mut Vec<CNum<'a, CS>>) {
        acc.push(self.clone());
    }

    #[inline]
    fn get_cs(&self) -> &'a CS {
        self.cs
    }

    fn as_const(&self) -> Option<Self::Value> {
        if self.lc.len()==0 {
            Some(Num::zero())
        } else if self.lc.len() == 1 {
            let front = self.lc.front().unwrap();
            if front.0 == Index::Input(0) {
                Some(front.1)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    fn from_const(cs:&'a CS, value: &Self::Value) -> Self {
        let mut lc = LinkedList::new();
        lc.push_back((Index::Input(0), value.clone()));
        let value = Some(value.clone());
        Self {value, lc, cs}
    }

    fn alloc(cs:&'a CS, value:Option<&Self::Value>) -> Self {
        let value = value.cloned();
        let var = cs.alloc(value);
        Self::from_var(cs, value, var)
    }

    fn assert_const(&self, c:&Self::Value) {
        match self.as_const() {
            Some(v) => {
                assert!(v==*c); 
            },
            _ => {
                self.cs.enforce(self, &self.derive_one(), &self.derive_const(c));
            }
        }
    }

    fn assert_eq(&self, other:&Self) {
        (self-other).assert_zero()
    }

    fn is_eq(&self, other:&Self) -> CBool<'a, CS>{
        (self-other).is_zero()
    }
}



impl<'a, CS:ConstraintSystem> CNum<'a, CS> {
    
    #[inline]
    pub fn capacity(&self) -> usize {
        self.lc.len()
    }


    #[inline]
    pub fn from_var(cs:&'a CS, value: Option<Num<CS::F>>, var: Index) -> Self {
        let mut lc = LinkedList::new();
        lc.push_back((var, Num::one()));
        Self {value, lc, cs}
    }

    #[inline]
    pub fn derive_var(&self, value: Option<Num<CS::F>>, var: Index) -> Self {
        CNum::from_var(self.cs, value, var)
    }


    #[inline]
    pub fn zero(cs:&'a CS) -> Self {
        Self::from_const(cs, &Num::zero())
    }

    #[inline]
    pub fn one(cs:&'a CS) -> Self {
        Self::from_const(cs, &Num::one())
    }

    #[inline]
    pub fn derive_zero(&self) -> Self {
        Self::zero(self.cs)
    }

    #[inline]
    pub fn derive_one(&self) -> Self {
        Self::one(self.cs)
    }

    #[inline]
    pub fn square(&self) -> Self {
        self * self
    }

    pub fn assert_zero(&self) {
        self.assert_const(&Num::zero());
    }

    pub fn assert_nonzero(&self) {
        match self.as_const() {
            Some(v) => {
                assert!(v!=Num::zero());
            },
            _ => {
                let inv_value = match self.get_value() {
                    Some(t) => if t.is_zero() {
                        Some(Num::one())
                    } else {
                        Some(t.inverse())
                    }
                    None => None
                };
                let inv_signal = self.derive_alloc(inv_value.as_ref());
                self.cs.enforce(self, &inv_signal, &self.derive_one());
            }
        }
    }

    pub fn into_bool(self) -> CBool<'a, CS> {
        CBool(self)
    }

    pub fn assert_bit(&self) {
        match self.as_const() {
            Some(c) => {
                assert!(c==Num::one() || c== Num::zero());
            },
            _ => {
                self.cs.enforce(self, &(self - Num::one()), &self.derive_zero());
            }
        }
    }

    pub fn is_zero(&self) -> CBool<'a, CS> {
        match self.as_const() {
            Some(c) => self.derive_const(&c.is_zero()),
            _ => {
                let inv_value = match self.get_value() {
                    Some(t) => if t.is_zero() {
                        Some(Num::one())
                    } else {
                        Some(t.inverse())
                    }
                    None => None
                };
                
                let inv_signal = self.derive_alloc::<CNum<_>>(inv_value.as_ref());
                inv_signal.assert_nonzero();

                let res_signal = inv_signal * self;
                res_signal.assert_bit();
                (self.derive_one() - res_signal).into_bool()
            }
        }
    }
}


impl<'a, CS:ConstraintSystem> Neg for CNum<'a, CS> {
    type Output = CNum<'a, CS>;
    fn neg(mut self) -> Self::Output {
        self.value = self.value.map(|x| -x);

        for (_, v) in self.lc.iter_mut() {
            *v = -*v;
        }
        self
    }
}

forward_unop_ex!(impl<'a, CS:ConstraintSystem> Neg for CNum<'a, CS>, neg);

#[derive(Eq,PartialEq)]
enum LookupAction {
    Add,
    Insert
}

#[inline]
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



impl<'l, 'a, CS:ConstraintSystem> AddAssign<&'l CNum<'a, CS>> for CNum<'a, CS> {
    #[inline]
    fn add_assign(&mut self, other: &'l CNum<CS>)  {
        self.value = match (self.get_value(), other.get_value()) {
            (Some(a), Some(b)) => Some(a+b),
            _ => None
        };

        let mut cur_a_ll = self.lc.cursor();

        for (k, v) in other.lc.iter() {
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
    }
}

impl<'l, 'a, CS:ConstraintSystem> AddAssign<&'l Num<CS::F>> for CNum<'a, CS> {
    #[inline]
    fn add_assign(&mut self, other: &'l Num<CS::F>)  {
        *self += self.derive_const::<Self>(other)
    }
}


impl<'l, 'a, CS:ConstraintSystem> SubAssign<&'l CNum<'a, CS>> for CNum<'a, CS> {

    #[inline]
    fn sub_assign(&mut self, other: &'l CNum<CS>)  {
        self.value = match (self.get_value(), other.get_value()) {
            (Some(a), Some(b)) => Some(a-b),
            _ => None
        };

        let mut cur_a_ll = self.lc.cursor();

        for (k, v) in other.lc.iter() {
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
    }
}

impl<'l, 'a, CS:ConstraintSystem> SubAssign<&'l Num<CS::F>> for CNum<'a, CS> {
    #[inline]
    fn sub_assign(&mut self, other: &'l Num<CS::F>)  {
        *self -= self.derive_const::<Self>(other)
    }
}


impl<'l, 'a, CS:ConstraintSystem> MulAssign<&'l Num<CS::F>> for CNum<'a, CS> {
    #[inline]
    fn mul_assign(&mut self, other: &'l Num<CS::F>)  {
        if other.is_zero() {
            *self = self.derive_zero()
        } else {
            self.value = self.value.map(|v| v*other);
            for (_, v) in self.lc.iter_mut() {
                *v *= other;
            }
        }
    }
}

impl<'l, 'a, CS:ConstraintSystem> DivAssign<&'l Num<CS::F>> for CNum<'a, CS> {
    #[inline]
    fn div_assign(&mut self, other: &'l Num<CS::F>)  {
        self.value = self.value.map(|v| v/other);
        for (_, v) in self.lc.iter_mut() {
            *v /= other;
        }
    }
}


forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> AddAssign<CNum<'a, CS>> for CNum<'a, CS>, add_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> SubAssign<CNum<'a, CS>> for CNum<'a, CS>, sub_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> AddAssign<Num<CS::F>> for CNum<'a, CS>, add_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> SubAssign<Num<CS::F>> for CNum<'a, CS>, sub_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> MulAssign<Num<CS::F>> for CNum<'a, CS>, mul_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> DivAssign<Num<CS::F>> for CNum<'a, CS>, div_assign);


impl<'l, 'a, CS:ConstraintSystem> Add<&'l CNum<'a, CS>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn add(mut self, other: &'l CNum<'a, CS>) -> Self::Output  {
        self += other;
        self
    }
}

impl<'l, 'a, CS:ConstraintSystem> Sub<&'l CNum<'a, CS>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn sub(mut self, other: &'l CNum<'a, CS>) -> Self::Output  {
        self -= other;
        self
    }
}


impl<'l, 'a, CS:ConstraintSystem> Add<&'l Num<CS::F>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn add(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self += CNum::from_const(self.cs, other);
        self
    }
}


impl<'l, 'a, CS:ConstraintSystem> Sub<&'l Num<CS::F>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn sub(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self -= CNum::from_const(self.cs, other);
        self
    }
}

impl<'l, 'a, CS:ConstraintSystem> Sub<&'l CNum<'a, CS>> for Num<CS::F> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn sub(self, other: &'l CNum<'a, CS>) -> Self::Output  {
        CNum::from_const(other.cs, &self) - other
    }
}


impl<'l, 'a, CS:ConstraintSystem> Mul<&'l Num<CS::F>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn mul(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self *= other;
        self
    }
}

impl<'l, 'a, CS:ConstraintSystem> Div<&'l Num<CS::F>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn div(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self /= other;
        self
    }
}

forward_all_binop_to_val_ref_commutative_ex!(impl<'a, CS:ConstraintSystem> Add for CNum<'a, CS>, add);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Sub<CNum<'a, CS>> for CNum<'a, CS>, sub -> CNum<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Add<Num<CS::F>> for CNum<'a, CS>, add -> CNum<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Sub<Num<CS::F>> for CNum<'a, CS>, sub -> CNum<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Sub<CNum<'a, CS>> for Num<CS::F>, sub -> CNum<'a, CS>);

forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Mul<Num<CS::F>> for CNum<'a, CS>, mul -> CNum<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Div<Num<CS::F>> for CNum<'a, CS>, div -> CNum<'a, CS>);

swap_commutative!(impl<'a, CS:ConstraintSystem> Add<Num<CS::F>> for CNum<'a, CS>, add);
swap_commutative!(impl<'a, CS:ConstraintSystem> Mul<Num<CS::F>> for CNum<'a, CS>, mul);

impl<'l, 'a, CS:ConstraintSystem> MulAssign<&'l CNum<'a, CS>> for CNum<'a, CS> {
    #[inline]
    fn mul_assign(&mut self, other: &'l CNum<'a, CS>)  {
        match (self.as_const(), other.as_const()) {
            (Some(a), _) => {*self = other*a;},
            (_, Some(b)) => {*self*=b;},
            _ => {
                let value = match(self.get_value(), other.get_value()) {
                    (Some(a), Some(b)) => Some(a*b),
                    _ => None
                };

                let a_mul_b = self.derive_alloc(value.as_ref());
                self.cs.enforce(self, other, &a_mul_b);
                *self = a_mul_b;
            }
        }
    }
}


impl<'l, 'a, CS:ConstraintSystem> DivAssign<&'l CNum<'a, CS>> for CNum<'a, CS> {
    #[inline]
    fn div_assign(&mut self, other: &'l CNum<'a, CS>)  {
        match (self.as_const(), other.as_const()) {
            (Some(a), _) => {*self = other/a; },
            (_, Some(b)) => {*self /= b},
            _ => {
                let value = match(self.get_value(), other.get_value()) {
                    (Some(a), Some(b)) => Some(a/b),
                    _ => None
                };


                let a_div_b = CNum::alloc(self.cs, value.as_ref());
                self.cs.enforce(&a_div_b, other, self);
                *self = a_div_b;
            }
        }
    }
}

forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> MulAssign<CNum<'a, CS>> for CNum<'a, CS>, mul_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> DivAssign<CNum<'a, CS>> for CNum<'a, CS>, div_assign);


impl<'l, 'a, CS:ConstraintSystem> Mul<&'l CNum<'a, CS>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn mul(mut self, other: &'l CNum<'a, CS>) -> Self::Output  {
        self *= other;
        self
    }
}


impl<'l, 'a, CS:ConstraintSystem> Div<&'l CNum<'a, CS>> for CNum<'a, CS> {
    type Output = CNum<'a, CS>;

    #[inline]
    fn div(mut self, other: &'l CNum<'a, CS>) -> Self::Output  {
        self /= other;
        self
    }
}

forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Mul<CNum<'a, CS>> for CNum<'a, CS>, mul -> CNum<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Div<CNum<'a, CS>> for CNum<'a, CS>, div -> CNum<'a, CS>);

#[cfg(test)]
mod num_test {
    use super::*;
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};


    #[test]
    fn add() {
        let mut rng = thread_rng();
        let ref cs = crate::core::cs::TestCS::<Fr>::new();
        let ref n_a = rng.gen();
        let ref n_b = rng.gen();

        let a = CNum::from_const(cs, n_a);
        let b = CNum::from_const(cs, n_b);
        let c = a+b;
        assert!(c.get_value().unwrap()==n_a+n_b);
    }

    #[test]
    fn add_mixed() {
        let mut rng = thread_rng();
        let ref cs = crate::core::cs::TestCS::<Fr>::new();
        let ref n_a = rng.gen();
        let ref n_b: Num<_> = rng.gen();

        let a = CNum::from_const(cs, n_a);
        let c = a+n_b;
        assert!(c.get_value().unwrap()==n_a+n_b);
    }

}