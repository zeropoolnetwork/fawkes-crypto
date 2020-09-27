use crate::circuit::{bool::CBool, cs::RCS};
use ff_uint::PrimeField;


pub trait Signal: Sized+Clone {
    type Value: Clone + Sized;
    type Fr:PrimeField;


    fn as_const(&self) -> Option<Self::Value>;

    fn get_value(&self) -> Option<Self::Value>;

    #[inline]
    fn derive_const<T:Signal<Fr=Self::Fr>>(&self, value: &T::Value) -> T {
        T::from_const(self.get_cs(), value)
    }

    fn from_const(cs:&RCS<Self::Fr>, value: &Self::Value) -> Self;

    fn get_cs(&self) -> &RCS<Self::Fr>;

    fn alloc(cs:&RCS<Self::Fr>, value:Option<&Self::Value>) -> Self;

    fn switch(&self, bit: &CBool<Self::Fr>, if_else: &Self) -> Self;

    fn assert_const(&self, value: &Self::Value);

    fn assert_eq(&self, other:&Self);

    fn is_eq(&self, other:&Self) -> CBool<Self::Fr>;

    fn inputize(&self);

    #[inline]
    fn derive_alloc<T:Signal<Fr=Self::Fr>>(&self, value:Option<&T::Value>) -> T {
        T::alloc(self.get_cs(), value)
    }
}