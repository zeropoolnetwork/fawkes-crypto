use super::bool::SignalBool;

pub trait Signal: Sized+Clone {
    type Value: Clone + Sized;
    type CS: Clone;
    type Bool:SignalBool;

    fn as_const(&self) -> Option<Self::Value>;

    fn get_value(&self) -> Option<Self::Value>;

    #[inline]
    fn derive_const<T:Signal<CS=Self::CS>>(&self, value: &T::Value) -> T {
        T::from_const(self.get_cs(), value)
    }

    fn from_const(cs:&Self::CS, value: &Self::Value) -> Self;

    fn get_cs(&self) -> &Self::CS;

    fn alloc(cs:&Self::CS, value:Option<&Self::Value>) -> Self;

    fn switch(&self, bit: &Self::Bool, if_else: &Self) -> Self;

    fn assert_const(&self, value: &Self::Value);

    fn assert_eq(&self, other:&Self);

    fn is_eq(&self, other:&Self) -> Self::Bool;

    fn inputize(&self);

    // fn linearize_builder(&self, acc: &mut Vec<CNum<'a, CS>>);

    // fn linearize(&self) -> Vec<CNum<'a, CS>> {
    //     let mut acc = Vec::new();
    //     self.linearize_builder(&mut acc);
    //     acc
    // }

    #[inline]
    fn derive_alloc<T:Signal<CS=Self::CS>>(&self, value:Option<&T::Value>) -> T {
        T::alloc(self.get_cs(), value)
    }
}