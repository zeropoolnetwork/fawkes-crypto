use ff_uint::{Num, PrimeField};
use crate::circuit::{
    cs::{CS, LC, RCS}, 
    bool::CBool
};
use crate::core::signal::Signal;
use std::cell::RefCell;
use std::rc::Rc;

use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign, SubAssign, MulAssign, DivAssign};
use linked_list::{LinkedList, Cursor};

#[derive(Clone, Debug)]
pub struct CNum<Fr:PrimeField> {
    pub value:Option<Num<Fr>>,
    // a*x + b
    pub lc: LC<Fr>,
    pub cs: Rc<RefCell<CS<Fr>>>
}

impl<Fr:PrimeField> CNum<Fr> {
    pub fn assert_zero(&self) {
        self.assert_const(&Num::ZERO) 
    }
    
    // for 0/0 uncertainty case any return value is valid
    pub fn div_unchecked(&self, other:&Self) -> Self {
        match (self.as_const(), other.as_const()) {
            (_, Some(b)) => {self / b},
            _ => {
                let value = self.value.map(|a| other.value.map(|b| a/b)).flatten();
                let signal = self.derive_alloc(value.as_ref());
                CS::enforce(&signal, other, self);
                signal
            }
        }
    }

    pub fn assert_nonzero(&self) {
        match self.as_const() {
            Some(v) => {
                assert!(v!=Num::ZERO);
            },
            _ => {
                let inv_value = self.get_value().map(|v| v.checked_inv().unwrap_or(Num::ONE));
                let inv_signal = self.derive_alloc(inv_value.as_ref());
                CS::enforce(self, &inv_signal, &self.derive_const(&Num::ONE));
            }
        }
    }

    pub fn is_zero(&self) -> CBool<Fr> {
        match self.as_const() {
            Some(c) => self.derive_const(&c.is_zero()),
            _ => {
                let inv_value = self.get_value().map(|v| v.checked_inv().unwrap_or(Num::ONE));
                let inv_signal: CNum<Fr> = self.derive_alloc(inv_value.as_ref());
                inv_signal.assert_nonzero();
                let res_signal = inv_signal * self;
                (Num::ONE - res_signal).to_bool()
            }
        }
    }


    pub fn assert_bit(&self) {
        CS::enforce(self, &(self-Num::ONE), &self.derive_const(&Num::ZERO));
    }

    pub fn to_bool(&self) -> CBool<Fr> {
        CBool::new(self)
    }

    pub fn to_bool_unchecked(&self) -> CBool<Fr> {
        CBool::new_unchecked(self)
    }

    pub fn from_bool(b:CBool<Fr>) -> Self {
        b.to_num()
    }

    pub fn inv(&self) -> Self {
        let one:Self = self.derive_const(&Num::ONE);
        one/self
    }

    #[inline]
    pub fn square(&self) -> Self {
        self * self
    }

}

impl<Fr:PrimeField> Signal<Fr> for CNum<Fr> {
    type Value = Num<Fr>;


    fn as_const(&self) -> Option<Self::Value> {
        if self.lc.0.len()==0 {
            Some(Num::ZERO)
        } else if self.lc.0.len() == 1 {
            let front = self.lc.0.front().unwrap();
            if front.1 == 0 {
                Some(front.0)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn inputize(&self) {
        CS::enforce_pub(&self);
    }

    fn get_value(&self) -> Option<Self::Value> {
        self.value
    }

    fn from_const(cs:&RCS<Fr>, value: &Self::Value) -> Self {
        let value = value.clone();
        let mut ll = LinkedList::new();
        ll.push_back((value, 0));
        Self {
            value: Some(value),
            lc: LC(ll),
            cs:cs.clone()
        }
    }

    fn get_cs(&self) -> &RCS<Fr> {
        &self.cs
    }

    fn alloc(cs:&RCS<Fr>, value:Option<&Self::Value>) -> Self {
        CS::alloc(cs, value)
    }

    fn assert_const(&self, value: &Self::Value) {
        CS::enforce(self, &self.derive_const(&Num::ONE), &self.derive_const(value))
    }

    fn switch(&self, bit: &CBool<Fr>, if_else: &Self) -> Self {
        if let Some(b) = bit.as_const() {
            if b {self.clone()} else {if_else.clone()}
        } else {
            if_else + (self - if_else) * bit.to_num()
        }
    }

    fn assert_eq(&self, other:&Self) {
        CS::enforce(self, &self.derive_const(&Num::ONE), other);
    }

    fn is_eq(&self, other:&Self) -> CBool<Fr> {
        (self-other).is_zero()
    }
    
}


impl<Fr:PrimeField> CNum<Fr> {
    pub fn capacity(&self) -> usize {
        self.lc.0.len()
    }

}


impl<Fr:PrimeField> std::ops::Neg for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn neg(mut self) -> Self::Output {
        self.value = self.value.map(|x| -x);
        for (v, _) in self.lc.0.iter_mut() {
            *v = -*v;
        }
        self
    }
}

forward_unop_ex!(impl<Fr:PrimeField> Neg for CNum<Fr>, neg);


#[derive(Eq,PartialEq)]
enum LookupAction {
    Add,
    Insert
}


#[inline]
fn ll_lookup<V, K:PartialEq+PartialOrd>(cur: &mut Cursor<(V, K)>, n: K) -> LookupAction {
    loop {
        match cur.peek_next() {
            Some((_, k)) => {
                if  *k == n {
                    return LookupAction::Add;
                } else if *k < n {
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



impl<'l, Fr:PrimeField> AddAssign<&'l CNum<Fr>> for CNum<Fr> {
    #[inline]
    fn add_assign(&mut self, other: &'l CNum<Fr>)  {
        self.value = self.value.map(|a| other.value.map(|b| a+b)).flatten();

        let mut cur_a_ll = self.lc.0.cursor();


        for (v, k) in other.lc.0.iter() {
            if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                let t = cur_a_ll.peek_next().unwrap();
                t.0 += *v;
                if t.0.is_zero() {
                    cur_a_ll.remove();
                }
            } else {
                cur_a_ll.insert((*v, *k))
            }
        }
    }
}

impl<'l, Fr:PrimeField> AddAssign<&'l Num<Fr>> for CNum<Fr> {
    #[inline]
    fn add_assign(&mut self, other: &'l Num<Fr>)  {
        *self += self.derive_const::<Self>(other)
    }
}


impl<'l, Fr:PrimeField> SubAssign<&'l CNum<Fr>> for CNum<Fr> {

    #[inline]
    fn sub_assign(&mut self, other: &'l CNum<Fr>)  {
        self.value = self.value.map(|a| other.value.map(|b| a-b)).flatten();

        let mut cur_a_ll = self.lc.0.cursor();

        for (v, k) in other.lc.0.iter() {
            if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                let t = cur_a_ll.peek_next().unwrap();
                t.0 -= *v;
                if t.0.is_zero() {
                    cur_a_ll.remove();
                }
            } else {
                cur_a_ll.insert((-*v, *k))
            }
        }
    }
}

impl<'l, Fr:PrimeField> SubAssign<&'l Num<Fr>> for CNum<Fr> {
    #[inline]
    fn sub_assign(&mut self, other: &'l Num<Fr>)  {
        *self -= self.derive_const::<Self>(other)
    }
}


impl<'l, Fr:PrimeField> MulAssign<&'l Num<Fr>> for CNum<Fr> {
    #[inline]
    fn mul_assign(&mut self, other: &'l Num<Fr>)  {
        if other.is_zero() {
            *self = self.derive_const(&Num::ZERO)
        } else {
            self.value = self.value.map(|v| v*other);
            for (v, _) in self.lc.0.iter_mut() {
                *v *= other;
            }
        }
    }
}

impl<'l, Fr:PrimeField> DivAssign<&'l Num<Fr>> for CNum<Fr> {
    #[inline]
    fn div_assign(&mut self, other: &'l Num<Fr>)  {
        let inv = other.checked_inv().expect("Division by zero");
        self.mul_assign(&inv);
    }
}


impl<'l, Fr:PrimeField> MulAssign<&'l CNum<Fr>> for CNum<Fr> {
    #[inline]
    fn mul_assign(&mut self, other: &'l CNum<Fr>)  {
        match (self.as_const(), other.as_const()) {
            (Some(a), _) => {*self = other*a;},
            (_, Some(b)) => {*self *= b;},
            _ => {
                let value = self.value.map(|a| other.value.map(|b| a*b)).flatten();

                let signal = self.derive_alloc(value.as_ref());
                CS::enforce(self, other, &signal);
                *self = signal;
            }
        }
    }
}


impl<'l, Fr:PrimeField> DivAssign<&'l CNum<Fr>> for CNum<Fr> {
    #[inline]
    fn div_assign(&mut self, other: &'l CNum<Fr>)  {
        other.assert_nonzero();
        *self = self.div_unchecked(other);
    }
}



forward_val_assign_ex!(impl<Fr:PrimeField> AddAssign<CNum<Fr>> for CNum<Fr>, add_assign);
forward_val_assign_ex!(impl<Fr:PrimeField> AddAssign<Num<Fr>> for CNum<Fr>, add_assign);

forward_val_assign_ex!(impl<Fr:PrimeField> SubAssign<CNum<Fr>> for CNum<Fr>, sub_assign);
forward_val_assign_ex!(impl<Fr:PrimeField> SubAssign<Num<Fr>> for CNum<Fr>, sub_assign);

forward_val_assign_ex!(impl<Fr:PrimeField> MulAssign<CNum<Fr>> for CNum<Fr>, mul_assign);
forward_val_assign_ex!(impl<Fr:PrimeField> MulAssign<Num<Fr>> for CNum<Fr>, mul_assign);

forward_val_assign_ex!(impl<Fr:PrimeField> DivAssign<CNum<Fr>> for CNum<Fr>, div_assign);
forward_val_assign_ex!(impl<Fr:PrimeField> DivAssign<Num<Fr>> for CNum<Fr>, div_assign);


impl<'l, Fr:PrimeField> Add<&'l CNum<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn add(mut self, other: &'l CNum<Fr>) -> Self::Output  {
        self += other;
        self
    }
}

impl<'l, Fr:PrimeField> Add<&'l Num<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn add(mut self, other: &'l Num<Fr>) -> Self::Output  {
        self += other;
        self
    }
}


impl<'l, Fr:PrimeField> Sub<&'l CNum<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn sub(mut self, other: &'l CNum<Fr>) -> Self::Output  {
        self -= other;
        self
    }
}


impl<'l, Fr:PrimeField> Sub<&'l Num<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn sub(mut self, other: &'l Num<Fr>) -> Self::Output  {
        self -= other;
        self
    }
}

impl<'l, Fr:PrimeField> Sub<&'l CNum<Fr>> for Num<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn sub(self, other: &'l CNum<Fr>) -> Self::Output  {
        -other+self
    }
}


impl<'l, Fr:PrimeField> Mul<&'l Num<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn mul(mut self, other: &'l Num<Fr>) -> Self::Output  {
        self *= other;
        self
    }
}

impl<'l, Fr:PrimeField> Mul<&'l CNum<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn mul(mut self, other: &'l CNum<Fr>) -> Self::Output  {
        self *= other;
        self
    }
}


impl<'l, Fr:PrimeField> Div<&'l CNum<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn div(mut self, other: &'l CNum<Fr>) -> Self::Output  {
        self /= other;
        self
    }
}


impl<'l, Fr:PrimeField> Div<&'l Num<Fr>> for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn div(mut self, other: &'l Num<Fr>) -> Self::Output  {
        self /= other;
        self
    }
}

impl<'l, Fr:PrimeField> Div<&'l CNum<Fr>> for Num<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn div(self, other: &'l CNum<Fr>) -> Self::Output  {
        other.inv()*self
    }
}




forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Sub<CNum<Fr>> for CNum<Fr>, sub -> CNum<Fr>);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Sub<CNum<Fr>> for Num<Fr>, sub -> CNum<Fr>);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Sub<Num<Fr>> for CNum<Fr>, sub -> CNum<Fr>);

forward_all_binop_to_val_ref_commutative_ex!(impl<Fr:PrimeField> Add for CNum<Fr>, add);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Add<Num<Fr>> for CNum<Fr>, add -> CNum<Fr>);
swap_commutative!(impl<Fr:PrimeField> Add<Num<Fr>> for CNum<Fr>, add);

forward_all_binop_to_val_ref_commutative_ex!(impl<Fr:PrimeField> Mul for CNum<Fr>, mul);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Mul<Num<Fr>> for CNum<Fr>, mul -> CNum<Fr>);
swap_commutative!(impl<Fr:PrimeField> Mul<Num<Fr>> for CNum<Fr>, mul);

forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Div<CNum<Fr>> for CNum<Fr>, div -> CNum<Fr>);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Div<Num<Fr>> for CNum<Fr>, div -> CNum<Fr>);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> Div<CNum<Fr>> for Num<Fr>, div -> CNum<Fr>);

