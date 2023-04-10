use crate::{
    circuit::{
        cs::{CS, RCS},
        num::CNum,
    },
    core::signal::Signal,
    ff_uint::{Num},
};

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Clone, Debug)]
pub struct CBool<C: CS>(CNum<C>);

impl<C: CS> CBool<C> {
    pub fn new_unchecked(n: &CNum<C>) -> Self {
        CBool(n.clone())
    }

    pub fn new(n: &CNum<C>) -> Self {
        n.assert_bit();
        Self::new_unchecked(n)
    }

    pub fn to_num(&self) -> CNum<C> {
        self.0.clone()
    }

    pub fn as_num(&self) -> &CNum<C> {
        &self.0
    }

    pub fn capacity(&self) -> usize {
        0
    }
}

impl<C: CS> Signal<C> for CBool<C> {
    type Value = bool;

    fn as_const(&self) -> Option<Self::Value> {
        let lc = self.0.lc;
        if lc.0 == Num::ZERO {
            if lc.2 == Num::ZERO {
                Some(false)
            } else if lc.2 == Num::ONE {
                Some(true)
            } else {
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
            if v == Num::ZERO {
                false
            } else if v == Num::ONE {
                true
            } else {
                panic!("Wrong boolean value")
            }
        })
    }

    fn from_const(cs: &RCS<C>, value: &Self::Value) -> Self {
        Self::new_unchecked(&CNum::from_const(cs, &(*value).into()))
    }

    fn get_cs(&self) -> &RCS<C> {
        &self.0.cs
    }

    fn alloc(cs: &RCS<C>, value: Option<&Self::Value>) -> Self {
        let value = value.map(|&b| Into::<Num<C::Fr>>::into(b));
        Self::new_unchecked(&CNum::alloc(cs, value.as_ref()))
    }

    fn assert_const(&self, value: &Self::Value) {
        C::enforce_add(
            &self.to_num(),
            &self.derive_const(&Num::ZERO),
            &self.derive_const(&(*value).into()),
        )
    }

    fn switch(&self, bit: &CBool<C>, if_else: &Self) -> Self {
        self.to_num()
            .switch(bit, &if_else.to_num())
            .to_bool_unchecked()
    }

    fn assert_eq(&self, other: &Self) {
        self.to_num().assert_eq(&other.to_num())
    }

    fn is_eq(&self, other: &Self) -> CBool<C> {
        let value = self
            .get_value()
            .map(|a| other.get_value().map(|b| a == b))
            .flatten();
        let signal: CBool<C> = self.derive_alloc(value.as_ref());
        C::enforce_mul(
            &(self.to_num() * Num::from(2) - Num::ONE),
            &(other.to_num() * Num::from(2) - Num::ONE),
            &(signal.to_num() * Num::from(2) - Num::ONE),
        );
        signal
    }
}

impl<C: CS> Not for CBool<C> {
    type Output = Self;

    fn not(self) -> Self::Output {
        (Num::ONE - self.to_num()).to_bool_unchecked()
    }
}

forward_unop_ex!(impl<C: CS> Not for CBool<C>, not);

impl<'l, C: CS> BitAndAssign<&'l CBool<C>> for CBool<C> {
    #[inline]
    fn bitand_assign(&mut self, other: &'l CBool<C>) {
        *self = (self.to_num() * other.to_num()).to_bool_unchecked()
    }
}

impl<'l, C: CS> BitAndAssign<&'l bool> for CBool<C> {
    #[inline]
    fn bitand_assign(&mut self, other: &'l bool) {
        *self &= self.derive_const::<Self>(other)
    }
}

impl<'l, C: CS> BitOrAssign<&'l CBool<C>> for CBool<C> {
    #[inline]
    fn bitor_assign(&mut self, other: &'l CBool<C>) {
        *self = !(!self.clone() & !other)
    }
}

impl<'l, C: CS> BitOrAssign<&'l bool> for CBool<C> {
    #[inline]
    fn bitor_assign(&mut self, other: &'l bool) {
        *self |= self.derive_const::<Self>(other)
    }
}

impl<'l, C: CS> BitXorAssign<&'l CBool<C>> for CBool<C> {
    #[inline]
    fn bitxor_assign(&mut self, other: &'l CBool<C>) {
        *self = !self.is_eq(other)
    }
}

impl<'l, C: CS> BitXorAssign<&'l bool> for CBool<C> {
    #[inline]
    fn bitxor_assign(&mut self, other: &'l bool) {
        *self ^= self.derive_const::<Self>(other)
    }
}

forward_val_assign_ex!(impl<C: CS> BitAndAssign<CBool<C>> for CBool<C>, bitand_assign);
forward_val_assign_ex!(impl<C: CS> BitAndAssign<bool> for CBool<C>, bitand_assign);

forward_val_assign_ex!(impl<C: CS> BitOrAssign<CBool<C>> for CBool<C>, bitor_assign);
forward_val_assign_ex!(impl<C: CS> BitOrAssign<bool> for CBool<C>, bitor_assign);

forward_val_assign_ex!(impl<C: CS> BitXorAssign<CBool<C>> for CBool<C>, bitxor_assign);
forward_val_assign_ex!(impl<C: CS> BitXorAssign<bool> for CBool<C>, bitxor_assign);

impl<'l, C: CS> BitAnd<&'l CBool<C>> for CBool<C> {
    type Output = CBool<C>;

    #[inline]
    fn bitand(mut self, other: &'l CBool<C>) -> Self::Output {
        self &= other;
        self
    }
}

impl<'l, C: CS> BitAnd<&'l bool> for CBool<C> {
    type Output = CBool<C>;

    #[inline]
    fn bitand(mut self, other: &'l bool) -> Self::Output {
        self &= other;
        self
    }
}

forward_all_binop_to_val_ref_commutative_ex!(impl<C: CS> BitAnd for CBool<C>, bitand);
forward_all_binop_to_val_ref_ex!(impl<C: CS> BitAnd<bool> for CBool<C>, bitand -> CBool<C>);
swap_commutative!(impl<C: CS> BitAnd<bool> for CBool<C>, bitand);

impl<'l, C: CS> BitOr<&'l CBool<C>> for CBool<C> {
    type Output = CBool<C>;

    #[inline]
    fn bitor(mut self, other: &'l CBool<C>) -> Self::Output {
        self |= other;
        self
    }
}

impl<'l, C: CS> BitOr<&'l bool> for CBool<C> {
    type Output = CBool<C>;

    #[inline]
    fn bitor(mut self, other: &'l bool) -> Self::Output {
        self |= other;
        self
    }
}

forward_all_binop_to_val_ref_commutative_ex!(impl<C: CS> BitOr for CBool<C>, bitor);
forward_all_binop_to_val_ref_ex!(impl<C: CS> BitOr<bool> for CBool<C>, bitor -> CBool<C>);
swap_commutative!(impl<C: CS> BitOr<bool> for CBool<C>, bitor);

impl<'l, C: CS> BitXor<&'l CBool<C>> for CBool<C> {
    type Output = CBool<C>;

    #[inline]
    fn bitxor(mut self, other: &'l CBool<C>) -> Self::Output {
        self ^= other;
        self
    }
}

impl<'l, C: CS> BitXor<&'l bool> for CBool<C> {
    type Output = CBool<C>;

    #[inline]
    fn bitxor(mut self, other: &'l bool) -> Self::Output {
        self ^= other;
        self
    }
}

forward_all_binop_to_val_ref_commutative_ex!(impl<C: CS> BitXor for CBool<C>, bitxor);
forward_all_binop_to_val_ref_ex!(impl<C: CS> BitXor<bool> for CBool<C>, bitxor -> CBool<C>);
swap_commutative!(impl<C: CS> BitXor<bool> for CBool<C>, bitxor);
