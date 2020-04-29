use std::marker::{Sized, PhantomData};
use typenum::Unsigned;


use crate::core::cs::ConstraintSystem;
use crate::circuit::bool::CBool;
use crate::circuit::num::CNum;
use crate::core::sizedvec::SizedVec;

pub trait Signal <'a, CS:'a+ConstraintSystem> : Sized+Clone {
    type Value: Clone + Sized;

    fn get_cs(&self) -> &'a CS;

    fn from_const(cs:&'a CS, value: &Self::Value) -> Self;
    
    fn get_value(&self) -> Option<Self::Value>;

    fn alloc(cs:&'a CS, value:Option<&Self::Value>) -> Self;

    fn switch(&self, bit: &CBool<'a, CS>, if_else: &Self) -> Self;

    fn assert_const(&self, value: &Self::Value);

    fn assert_eq(&self, other:&Self);

    fn is_eq(&self, other:&Self) -> CBool<'a, CS>;


    #[inline]
    fn as_const(&self) -> Option<Self::Value> { None }

    #[inline]
    fn derive_const<T:Signal<'a, CS>>(&self, value: &T::Value) -> T {
        T::from_const(self.get_cs(), value)
    }

    #[inline]
    fn derive_alloc<T:Signal<'a, CS>>(&self, value:Option<&T::Value>) -> T {
        T::alloc(self.get_cs(), value)
    }
}


impl <'a, CS:'a+ConstraintSystem, T:Signal<'a, CS>, L:Unsigned> Signal<'a, CS> for SizedVec<T, L> {
    type Value = SizedVec<T::Value, L>;

    fn get_value(&self) -> Option<Self::Value> {
        self.iter().map(|v| v.get_value()).collect()
    }

    fn switch(&self, bit: &CBool<'a, CS>, if_else: &Self) -> Self {
        self.iter().zip(if_else.iter()).map(|(t,f)| t.switch(bit, f)).collect()
    }


    fn get_cs(&self) -> &'a CS {
        self[0].get_cs()
    }

    fn from_const(cs:&'a CS, value: &Self::Value) -> Self {
        value.iter().map(|v| T::from_const(cs, v)).collect()
    }


    fn alloc(cs:&'a CS, value:Option<&Self::Value>) -> Self {
        match value {
            Some(value) => value.iter().map(|v| T::alloc(cs, Some(v))).collect(),
            _ =>  SizedVec(vec![T::alloc(cs, None); L::USIZE], PhantomData) 
        }
    }

    fn assert_const(&self, value: &Self::Value) {
        self.iter().zip(value.iter()).for_each(|(s, v)| s.assert_const(v));
    }
    
    fn assert_eq(&self, other: &Self) {
        self.iter().zip(other.iter()).for_each(|(s, o)| s.assert_eq(o));
    }

    fn is_eq(&self, other:&Self) -> CBool<'a, CS> {
        let mut acc = CNum::one(self.get_cs());
        for i in 0..L::USIZE {
            acc *= self[i].is_eq(&other[i]).0;
        }
        acc.into_bool()
    }


}




// seq!(N in 1..=1 {
//     impl <'a, CS:'a+ConstraintSystem, T:Signal<'a, CS>> Signal<'a, CS> for [T;N]{
//         type Value = [T::Value;N];

//         fn get_value(&self) -> Option<Self::Value> {
//             collect_opt_array!([T::Value; N], self.iter().map(|v| v.get_value()))
//         }

//         fn switch(&self, bit: &CNum<'a, CS>, if_else: &Self) -> Self {
//             collect_array!([T;N], self.iter().zip(if_else.iter()).map(|(t,f)| t.switch(bit, f)))
//         }


//         fn get_cs(&self) -> &'a CS {
//             self[0].get_cs()
//         }

//         fn from_const(cs:&'a CS, value: &Self::Value) -> Self {
//             collect_array!([T;N], value.iter().map(|v| T::from_const(cs, v)))
//         }


//         fn alloc(cs:&'a CS, value:Option<&Self::Value>) -> Self {
//             match value {
//                 Some(value) => collect_array!([T;N], value.iter().map(|v| T::alloc(cs, Some(v)))),
//                 _ => collect_array!([T;N], (0..N).map(|_| T::alloc(cs, None)))
//             }
//         }

//     }
// });