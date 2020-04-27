use std::marker::Sized;
use seq_macro::seq;

use crate::core::cs::ConstraintSystem;
use crate::core::signal::Signal;

pub trait AbstractSignal <'a, CS:'a+ConstraintSystem> : Sized {
    type Value: Clone + Sized;

    fn get_cs(&self) -> &'a CS;

    fn from_const(cs:&'a CS, value: Self::Value) -> Self;
    
    fn get_value(&self) -> Option<Self::Value>;

    fn as_const(&self) -> Option<Self::Value> { None }

    fn alloc(cs:&'a CS, value:Option<Self::Value>) -> Self;

    fn switch(&self, bit: &Signal<'a, CS>, if_else: &Self) -> Self;

    #[inline]
    fn derive_const<T:AbstractSignal<'a, CS>>(&self, value: T::Value) -> T {
        T::from_const(self.get_cs(), value)
    }

    #[inline]
    fn derive_alloc<T:AbstractSignal<'a, CS>>(&self, value:Option<T::Value>) -> T {
        T::alloc(self.get_cs(), value)
    }
}



seq!(N in 1..=1 {
    impl <'a, CS:'a+ConstraintSystem, T:AbstractSignal<'a, CS>> AbstractSignal<'a, CS> for [T;N]{
        type Value = [T::Value;N];

        fn get_value(&self) -> Option<Self::Value> {
            collect_opt_array!([T::Value; N], self.iter().map(|v| v.get_value()))
        }

        fn switch(&self, bit: &Signal<'a, CS>, if_else: &Self) -> Self {
            collect_array!([T;N], self.iter().zip(if_else.iter()).map(|(t,f)| t.switch(bit, f)))
        }


        fn get_cs(&self) -> &'a CS {
            self[0].get_cs()
        }

        fn from_const(cs:&'a CS, value: Self::Value) -> Self {
            collect_array!([T;N], value.iter().map(|v| T::from_const(cs, v.clone())))
        }


        fn alloc(cs:&'a CS, value:Option<Self::Value>) -> Self {
            match value {
                Some(value) => collect_array!([T;N], value.iter().map(|v| T::alloc(cs, Some(v.clone())))),
                _ => collect_array!([T;N], (0..N).map(|_| T::alloc(cs, None)))
            }
        }

    }
});