use ff_uint::{Num, PrimeField};
use crate::circuit::r1cs::{num::CNum, cs::CS};
use crate::circuit::general::{traits::{signal::Signal, bool::SignalBool, num::SignalNum}};
use std::cell::RefCell;
use std::rc::Rc;

use std::ops::{Not, BitAndAssign, BitOrAssign, BitXorAssign, BitAnd, BitOr, BitXor};

#[derive(Clone, Debug)]
pub struct CBool<Fr:PrimeField>(CNum<Fr>);

impl<Fr:PrimeField> SignalBool for CBool<Fr> {
    type Num=CNum<Fr>;

    fn new_unchecked(n:&CNum<Fr>) -> Self {
        CBool(n.clone())
    }

    fn new(n: &CNum<Fr>) -> Self {
        n.assert_bit();
        Self::new_unchecked(n)
    }

    fn to_num(&self) -> CNum<Fr> {
        self.0.clone()
    }
}

impl<Fr:PrimeField> Signal for CBool<Fr> {
    type Value = bool;
    type CS = Rc<RefCell<CS<Fr>>>;
    type Bool = Self;

    fn as_const(&self) -> Option<Self::Value> {
        let lc = &self.0.lc;
        if lc.1.len() == 0 {
            if lc.0==Num::ZERO {
                Some(false)
            } else if lc.0==Num::ONE {
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
        let value = value.map(|&b| Into::<Num<Fr>>::into(b));
        Self::new_unchecked(&CNum::alloc(cs, value.as_ref()))
    }

    fn assert_const(&self, value: &Self::Value) {
        CS::enforce(&self.to_num(), &self.derive_const(&Num::ONE), &self.derive_const(&(*value).into()))
    }

    fn switch(&self, bit: &Self::Bool, if_else: &Self) -> Self {
        self.to_num().switch(bit, &if_else.to_num()).to_bool_unchecked()
    }

    fn assert_eq(&self, other:&Self) {
        self.to_num().assert_eq(&other.to_num())
    }

    fn is_eq(&self, other:&Self) -> Self::Bool {
        let value = self.get_value().map(|a| other.get_value().map(|b| a==b )).flatten();
        let signal:Self::Bool = self.derive_alloc(value.as_ref());
        CS::enforce(&(self.to_num()*Num::from(2)-Num::ONE), &(other.to_num()*Num::from(2)-Num::ONE), &(signal.to_num()*Num::from(2)-Num::ONE));
        signal
    }

}

impl<Fr:PrimeField> CBool<Fr> {
    pub fn capacity(&self) -> usize { 0 }
}

impl<Fr:PrimeField> Not for CBool<Fr> {
    type Output = Self;

    fn not(self) -> Self::Output {
        (Num::ONE-self.to_num()).to_bool_unchecked()
    }
}

forward_unop_ex!(impl<Fr:PrimeField> Not for CBool<Fr>, not);


impl<'l, Fr:PrimeField> BitAndAssign<&'l CBool<Fr>> for CBool<Fr> {
    #[inline]
    fn bitand_assign(&mut self, other: &'l CBool<Fr>)  {
        *self = (self.to_num()*other.to_num()).to_bool_unchecked()
    }
}

impl<'l, Fr:PrimeField> BitAndAssign<&'l bool> for CBool<Fr> {

    #[inline]
    fn bitand_assign(&mut self, other: &'l bool)  {
        *self &= self.derive_const::<Self>(other)
    }
}


impl<'l, Fr:PrimeField> BitOrAssign<&'l CBool<Fr>> for CBool<Fr> {
    #[inline]
    fn bitor_assign(&mut self, other: &'l CBool<Fr>)  {
        *self = !(!self.clone() & !other)
    }
}

impl<'l, Fr:PrimeField> BitOrAssign<&'l bool> for CBool<Fr> {
    #[inline]
    fn bitor_assign(&mut self, other: &'l bool)  {
        *self |= self.derive_const::<Self>(other)
    }
}



impl<'l, Fr:PrimeField> BitXorAssign<&'l CBool<Fr>> for CBool<Fr> {
    #[inline]
    fn bitxor_assign(&mut self, other: &'l CBool<Fr>)  {
        *self = !self.is_eq(other)
    }
}

impl<'l, Fr:PrimeField> BitXorAssign<&'l bool> for CBool<Fr> {
    #[inline]
    fn bitxor_assign(&mut self, other: &'l bool)  {
        *self ^= self.derive_const::<Self>(other)
    }
}



forward_val_assign_ex!(impl<Fr:PrimeField> BitAndAssign<CBool<Fr>> for CBool<Fr>, bitand_assign);
forward_val_assign_ex!(impl<Fr:PrimeField> BitAndAssign<bool> for CBool<Fr>, bitand_assign);

forward_val_assign_ex!(impl<Fr:PrimeField> BitOrAssign<CBool<Fr>> for CBool<Fr>, bitor_assign);
forward_val_assign_ex!(impl<Fr:PrimeField> BitOrAssign<bool> for CBool<Fr>, bitor_assign);

forward_val_assign_ex!(impl<Fr:PrimeField> BitXorAssign<CBool<Fr>> for CBool<Fr>, bitxor_assign);
forward_val_assign_ex!(impl<Fr:PrimeField> BitXorAssign<bool> for CBool<Fr>, bitxor_assign);


impl<'l, Fr:PrimeField> BitAnd<&'l CBool<Fr>> for CBool<Fr> {
    type Output = CBool<Fr>;

    #[inline]
    fn bitand(mut self, other: &'l CBool<Fr>) -> Self::Output  {
        self &= other;
        self
    }
}

impl<'l, Fr:PrimeField> BitAnd<&'l bool> for CBool<Fr> {
    type Output = CBool<Fr>;

    #[inline]
    fn bitand(mut self, other: &'l bool) -> Self::Output  {
        self &= other;
        self
    }
}


forward_all_binop_to_val_ref_commutative_ex!(impl<Fr:PrimeField> BitAnd for CBool<Fr>, bitand);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> BitAnd<bool> for CBool<Fr>, bitand -> CBool<Fr>);
swap_commutative!(impl<Fr:PrimeField> BitAnd<bool> for CBool<Fr>, bitand);


impl<'l, Fr:PrimeField> BitOr<&'l CBool<Fr>> for CBool<Fr> {
    type Output = CBool<Fr>;

    #[inline]
    fn bitor(mut self, other: &'l CBool<Fr>) -> Self::Output  {
        self |= other;
        self
    }
}

impl<'l, Fr:PrimeField> BitOr<&'l bool> for CBool<Fr> {
    type Output = CBool<Fr>;

    #[inline]
    fn bitor(mut self, other: &'l bool) -> Self::Output  {
        self |= other;
        self
    }
}


forward_all_binop_to_val_ref_commutative_ex!(impl<Fr:PrimeField> BitOr for CBool<Fr>, bitor);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> BitOr<bool> for CBool<Fr>, bitor -> CBool<Fr>);
swap_commutative!(impl<Fr:PrimeField> BitOr<bool> for CBool<Fr>, bitor);


impl<'l, Fr:PrimeField> BitXor<&'l CBool<Fr>> for CBool<Fr> {
    type Output = CBool<Fr>;

    #[inline]
    fn bitxor(mut self, other: &'l CBool<Fr>) -> Self::Output  {
        self |= other;
        self
    }
}

impl<'l, Fr:PrimeField> BitXor<&'l bool> for CBool<Fr> {
    type Output = CBool<Fr>;

    #[inline]
    fn bitxor(mut self, other: &'l bool) -> Self::Output  {
        self |= other;
        self
    }
}


forward_all_binop_to_val_ref_commutative_ex!(impl<Fr:PrimeField> BitXor for CBool<Fr>, bitxor);
forward_all_binop_to_val_ref_ex!(impl<Fr:PrimeField> BitXor<bool> for CBool<Fr>, bitxor -> CBool<Fr>);
swap_commutative!(impl<Fr:PrimeField> BitXor<bool> for CBool<Fr>, bitxor);