use crate::{
    circuit::{
        bool::CBool,
        cs::{CS, RCS},
        bitify::c_into_bits_le_strict,
    },
    core::signal::Signal,
    ff_uint::{Num},
};
use std::{
    cell::RefCell,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct CNum<C: CS> {
    pub value: Option<Num<C::Fr>>,
    // a*x + b
    pub lc: (Num<C::Fr>, usize, Num<C::Fr>),
    pub cs: Rc<RefCell<C>>,
}

impl<C: CS> CNum<C> {
    pub fn assert_zero(&self) {
        self.assert_const(&Num::ZERO)
    }

    // FIXME: I'm not sure how this thing works. I copied it from R1CS.
    pub fn assert_even(&self) {
        let bits = c_into_bits_le_strict(&self);
        bits[0].assert_const(&false);
    }

    // for 0/0 uncertainty case any return value is valid
    pub fn div_unchecked(&self, other: &CNum<C>) -> Self {
        match (self.as_const(), other.as_const()) {
            (_, Some(b)) => self / b,
            _ => {
                let value = self.value.map(|a| other.value.map(|b| a / b)).flatten();
                let signal = self.derive_alloc(value.as_ref());
                C::enforce_mul(&signal, other, self);
                signal
            }
        }
    }

    pub fn assert_nonzero(&self) {
        match self.as_const() {
            Some(v) => {
                assert!(v != Num::ZERO);
            }
            _ => {
                let inv_value = self
                    .get_value()
                    .map(|v| v.checked_inv().unwrap_or(Num::ONE));
                let inv_signal = self.derive_alloc(inv_value.as_ref());
                C::enforce_mul(self, &inv_signal, &self.derive_const(&Num::ONE));
            }
        }
    }

    pub fn is_zero(&self) -> CBool<C> {
        match self.as_const() {
            Some(c) => self.derive_const(&c.is_zero()),
            _ => {
                let inv_value = self
                    .get_value()
                    .map(|v| v.checked_inv().unwrap_or(Num::ONE));
                let inv_signal: CNum<C> = self.derive_alloc(inv_value.as_ref());
                inv_signal.assert_nonzero();
                let res_signal = inv_signal * self;
                (Num::ONE - res_signal).to_bool()
            }
        }
    }

    pub fn assert_bit(&self) {
        C::enforce_mul(self, &(self - Num::ONE), &self.derive_const(&Num::ZERO));
    }

    pub fn to_bool(&self) -> CBool<C> {
        CBool::new(self)
    }

    pub fn to_bool_unchecked(&self) -> CBool<C> {
        CBool::new_unchecked(self)
    }

    pub fn from_bool(b: CBool<C>) -> Self {
        b.to_num()
    }

    pub fn inv(&self) -> Self {
        let one: Self = self.derive_const(&Num::ONE);
        one / self
    }

    #[inline]
    pub fn square(&self) -> Self {
        self * self
    }
}

impl<C: CS> Signal<C> for CNum<C> {
    type Value = Num<C::Fr>;

    fn as_const(&self) -> Option<Self::Value> {
        let lc = self.lc;
        if lc.0 == Num::ZERO {
            Some(lc.2)
        } else {
            None
        }
    }

    fn inputize(&self) {
        CS::inputize(&self);
    }

    fn get_value(&self) -> Option<Self::Value> {
        self.value
    }

    fn from_const(cs: &RCS<C>, value: &Self::Value) -> Self {
        let value = value.clone();
        Self {
            value: Some(value),
            lc: (Num::ZERO, 0, value),
            cs: cs.clone(),
        }
    }

    fn get_cs(&self) -> &RCS<C> {
        &self.cs
    }

    fn alloc(cs: &RCS<C>, value: Option<&Self::Value>) -> Self {
        CS::alloc(cs, value)
    }

    fn assert_const(&self, value: &Self::Value) {
        CS::enforce_add(
            self,
            &self.derive_const(&Num::ZERO),
            &self.derive_const(value),
        )
    }

    fn switch(&self, bit: &CBool<C>, if_else: &Self) -> Self {
        if let Some(b) = bit.as_const() {
            if b {
                self.clone()
            } else {
                if_else.clone()
            }
        } else {
            if_else + (self - if_else) * bit.to_num()
        }
    }

    fn assert_eq(&self, other: &Self) {
        CS::enforce_add(self, &self.derive_const(&Num::ZERO), other);
    }

    fn is_eq(&self, other: &Self) -> CBool<C> {
        (self - other).is_zero()
    }
}

impl<C: CS> CNum<C> {
    pub fn capacity(&self) -> usize {
        if self.lc.0 == Num::ZERO {
            0
        } else {
            1
        }
    }
}

impl<C: CS> std::ops::Neg for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn neg(mut self) -> Self::Output {
        self.lc.0 = -self.lc.0;
        self.lc.2 = -self.lc.2;
        self.value = self.value.map(|v| -v);
        self
    }
}

forward_unop_ex!(impl<C: CS> Neg for CNum<C>, neg);

impl<'l, C: CS> AddAssign<&'l CNum<C>> for CNum<C> {
    #[inline]
    fn add_assign(&mut self, other: &'l CNum<C>) {
        let cs = self.cs.clone();
        *self = if let Some(c) = self.as_const() {
            let value = other.value.map(|v| v + c);
            let mut lc = other.lc;
            lc.2 += c;
            Self { value, lc, cs }
        } else if let Some(c) = other.as_const() {
            let value = self.value.map(|v| v + c);
            let mut lc = self.lc;
            lc.2 += c;
            Self { value, lc, cs }
        } else if self.lc.1 == other.lc.1 {
            Self {
                value: self.value.map(|a| other.value.map(|b| a + b)).flatten(),
                lc: (self.lc.0 + other.lc.0, self.lc.1, self.lc.2 + other.lc.2),
                cs,
            }
        } else {
            let value = self.value.map(|a| other.value.map(|b| a + b)).flatten();
            let var: Self = self.derive_alloc(value.as_ref());
            CS::enforce_add(self, other, &var);
            var
        }
    }
}

impl<'l, C: CS> AddAssign<&'l Num<C::Fr>> for CNum<C> {
    #[inline]
    fn add_assign(&mut self, other: &'l Num<C::Fr>) {
        *self += self.derive_const::<Self>(other)
    }
}

impl<'l, C: CS> SubAssign<&'l CNum<C>> for CNum<C> {
    #[inline]
    fn sub_assign(&mut self, other: &'l CNum<C>) {
        self.add_assign(&-other)
    }
}

impl<'l, C: CS> SubAssign<&'l Num<C::Fr>> for CNum<C> {
    #[inline]
    fn sub_assign(&mut self, other: &'l Num<C::Fr>) {
        *self -= self.derive_const::<Self>(other)
    }
}

impl<'l, C: CS> MulAssign<&'l Num<C::Fr>> for CNum<C> {
    #[inline]
    fn mul_assign(&mut self, other: &'l Num<C::Fr>) {
        self.lc.0 *= other;
        self.lc.2 *= other;
        self.value = self.value.map(|v| v * other);
    }
}

impl<'l, C: CS> DivAssign<&'l Num<C::Fr>> for CNum<C> {
    #[inline]
    fn div_assign(&mut self, other: &'l Num<C::Fr>) {
        let inv = other.checked_inv().expect("Division by zero");
        self.mul_assign(&inv);
    }
}

impl<'l, C: CS> MulAssign<&'l CNum<C>> for CNum<C> {
    #[inline]
    fn mul_assign(&mut self, other: &'l CNum<C>) {
        let cs = self.cs.clone();
        *self = if let Some(c) = self.as_const() {
            let mut lc = other.lc;
            lc.0 *= c;
            lc.2 *= c;
            Self {
                value: other.value.map(|v| v * c),
                lc,
                cs,
            }
        } else if let Some(c) = other.as_const() {
            let mut lc = self.lc;
            lc.0 *= c;
            lc.2 *= c;
            Self {
                value: self.value.map(|v| v * c),
                lc,
                cs,
            }
        } else {
            let value = self.value.map(|a| other.value.map(|b| a * b)).flatten();
            let var = self.derive_alloc(value.as_ref());
            CS::enforce_mul(self, other, &var);
            var
        }
    }
}

impl<'l, C: CS> DivAssign<&'l CNum<C>> for CNum<C> {
    #[inline]
    fn div_assign(&mut self, other: &'l CNum<C>) {
        other.assert_nonzero();
        *self = self.div_unchecked(other);
    }
}

forward_val_assign_ex!(impl<C: CS> AddAssign<CNum<C>> for CNum<C>, add_assign);
forward_val_assign_ex!(impl<C: CS> AddAssign<Num<C::Fr>> for CNum<C>, add_assign);

forward_val_assign_ex!(impl<C: CS> SubAssign<CNum<C>> for CNum<C>, sub_assign);
forward_val_assign_ex!(impl<C: CS> SubAssign<Num<C::Fr>> for CNum<C>, sub_assign);

forward_val_assign_ex!(impl<C: CS> MulAssign<CNum<C>> for CNum<C>, mul_assign);
forward_val_assign_ex!(impl<C: CS> MulAssign<Num<C::Fr>> for CNum<C>, mul_assign);

forward_val_assign_ex!(impl<C: CS> DivAssign<CNum<C>> for CNum<C>, div_assign);
forward_val_assign_ex!(impl<C: CS> DivAssign<Num<C::Fr>> for CNum<C>, div_assign);

impl<'l, C: CS> Add<&'l CNum<C>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn add(mut self, other: &'l CNum<C>) -> Self::Output {
        self += other;
        self
    }
}

impl<'l, C: CS> Add<&'l Num<C::Fr>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn add(mut self, other: &'l Num<C::Fr>) -> Self::Output {
        self += other;
        self
    }
}

impl<'l, C: CS> Sub<&'l CNum<C>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn sub(mut self, other: &'l CNum<C>) -> Self::Output {
        self -= other;
        self
    }
}

impl<'l, C: CS> Sub<&'l Num<C::Fr>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn sub(mut self, other: &'l Num<C::Fr>) -> Self::Output {
        self -= other;
        self
    }
}

impl<'l, C: CS> Sub<&'l CNum<C>> for Num<C::Fr> {
    type Output = CNum<C>;

    #[inline]
    fn sub(self, other: &'l CNum<C>) -> Self::Output {
        -other + self
    }
}

impl<'l, C: CS> Mul<&'l Num<C::Fr>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn mul(mut self, other: &'l Num<C::Fr>) -> Self::Output {
        self *= other;
        self
    }
}

impl<'l, C: CS> Mul<&'l CNum<C>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn mul(mut self, other: &'l CNum<C>) -> Self::Output {
        self *= other;
        self
    }
}

impl<'l, C: CS> Div<&'l CNum<C>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn div(mut self, other: &'l CNum<C>) -> Self::Output {
        self /= other;
        self
    }
}

impl<'l, C: CS> Div<&'l Num<C::Fr>> for CNum<C> {
    type Output = CNum<C>;

    #[inline]
    fn div(mut self, other: &'l Num<C::Fr>) -> Self::Output {
        self /= other;
        self
    }
}

impl<'l, C: CS> Div<&'l CNum<C>> for Num<C::Fr> {
    type Output = CNum<C>;

    #[inline]
    fn div(self, other: &'l CNum<C>) -> Self::Output {
        other.inv() * self
    }
}

forward_all_binop_to_val_ref_ex!(impl<C: CS> Sub<CNum<C>> for CNum<C>, sub -> CNum<C>);
forward_all_binop_to_val_ref_ex!(impl<C: CS> Sub<CNum<C>> for Num<C::Fr>, sub -> CNum<C>);
forward_all_binop_to_val_ref_ex!(impl<C: CS> Sub<Num<C::Fr>> for CNum<C>, sub -> CNum<C>);

forward_all_binop_to_val_ref_commutative_ex!(impl<C: CS> Add for CNum<C>, add);
forward_all_binop_to_val_ref_ex!(impl<C: CS> Add<Num<C::Fr>> for CNum<C>, add -> CNum<C>);
swap_commutative!(impl<C: CS> Add<Num<C::Fr>> for CNum<C>, add);

forward_all_binop_to_val_ref_commutative_ex!(impl<C: CS> Mul for CNum<C>, mul);
forward_all_binop_to_val_ref_ex!(impl<C: CS> Mul<Num<C::Fr>> for CNum<C>, mul -> CNum<C>);
swap_commutative!(impl<C: CS> Mul<Num<C::Fr>> for CNum<C>, mul);

forward_all_binop_to_val_ref_ex!(impl<C: CS> Div<CNum<C>> for CNum<C>, div -> CNum<C>);
forward_all_binop_to_val_ref_ex!(impl<C: CS> Div<Num<C::Fr>> for CNum<C>, div -> CNum<C>);
forward_all_binop_to_val_ref_ex!(impl<C: CS> Div<CNum<C>> for Num<C::Fr>, div -> CNum<C>);
