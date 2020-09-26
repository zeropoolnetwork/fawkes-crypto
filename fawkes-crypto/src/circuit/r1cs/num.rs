use ff_uint::{Num, PrimeField};
use crate::circuit::{
    general::Variable,
    r1cs::{cs::{CS, LC}, bool::CBool},
    general::traits::{signal::Signal}
};
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
        match self.as_const() {
            Some(v) => {
                self.derive_const(&v.checked_inv().expect("Division by zero"))
            }
            _ => {
                self.assert_nonzero();
                let inv_value = self.get_value().map(|v| v.checked_inv().expect("Division by zero"));
                let inv_signal = self.derive_alloc(inv_value.as_ref());
                CS::enforce(self, &inv_signal, &self.derive_const(&Num::ONE));
                inv_signal
            }
        }
    }
}

impl<Fr:PrimeField> Signal for CNum<Fr> {
    type Value = Num<Fr>;
    type Fr = Fr;
    type CS = Rc<RefCell<CS<Fr>>>;

    fn as_const(&self) -> Option<Self::Value> {
        let lc = &self.lc;
        if lc.1.len() == 0 {
            Some(lc.0)
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

    fn from_const(cs:&Self::CS, value: &Self::Value) -> Self {
        let value = value.clone();
        Self {
            value: Some(value),
            lc: LC(value, LinkedList::new()),
            cs:cs.clone()
        }
    }

    fn get_cs(&self) -> &Self::CS {
        &self.cs
    }

    fn alloc(cs:&Self::CS, value:Option<&Self::Value>) -> Self {
        let mut rcs = cs.borrow_mut();
        let v = Variable(rcs.n_vars);
        rcs.n_vars+=1;
        let mut ll = LinkedList::new();
        ll.push_back((Num::ONE, v));
        Self {value:value.cloned(), lc:LC(Num::ZERO, ll), cs:cs.clone()}
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
        if self.lc.0==Num::ZERO {
            0
        } else {
            1
        }
    }

}


impl<Fr:PrimeField> std::ops::Neg for CNum<Fr> {
    type Output = CNum<Fr>;

    #[inline]
    fn neg(mut self) -> Self::Output {
        self.value = self.value.map(|x| -x);
        self.lc.0 = -self.lc.0;
        for (v, _) in self.lc.1.iter_mut() {
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



impl<'l, Fr:PrimeField> AddAssign<&'l CNum<Fr>> for CNum<Fr> {
    #[inline]
    fn add_assign(&mut self, other: &'l CNum<Fr>)  {
        self.value = self.value.map(|a| other.value.map(|b| a+b)).flatten();

        let mut cur_a_ll = self.lc.1.cursor();

        self.lc.0 += other.lc.0;

        for (v, k) in other.lc.1.iter() {
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
        self.value = self.value.map(|a| other.value.map(|b| a+b)).flatten();

        let mut cur_a_ll = self.lc.1.cursor();

        self.lc.0 -= other.lc.0;

        for (v, k) in other.lc.1.iter() {
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
            self.lc.0 *= other;
            for (v, _) in self.lc.1.iter_mut() {
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
        match (self.as_const(), other.as_const()) {
            (Some(a), _) => {*self = a*other.inv(); },
            (_, Some(b)) => {*self /= b},
            _ => {
                let value = self.value.map(|a| other.value.map(|b| a/b)).flatten();
                let signal = self.derive_alloc(value.as_ref());
                other.assert_nonzero();
                CS::enforce(&signal, other, self);
                *self = signal;
            }
        }
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

