use std::marker::{Sized};
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

    fn inputize(&self);

    fn linearize_builder(&self, acc: &mut Vec<CNum<'a, CS>>);

    fn as_const(&self) -> Option<Self::Value>;

    fn linearize(&self) -> Vec<CNum<'a, CS>> {
        let mut acc = Vec::new();
        self.linearize_builder(&mut acc);
        acc
    }

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

    fn as_const(&self) -> Option<Self::Value> {
        self.iter().map(|v| v.as_const()).collect()
    }


    fn alloc(cs:&'a CS, value:Option<&Self::Value>) -> Self {
        match value {
            Some(value) => value.iter().map(|v| T::alloc(cs, Some(v))).collect(),
            _ =>  (0..L::USIZE).map(|_| T::alloc(cs, None)).collect()
        }
    }

    fn assert_const(&self, value: &Self::Value) {
        self.iter().zip(value.iter()).for_each(|(s, v)| s.assert_const(v));
    }

    fn inputize(&self) {
        self.iter().for_each(|s| s.inputize());
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


    fn linearize_builder(&self, acc: &mut Vec<CNum<'a, CS>>) {
        self.iter().for_each(|s| s.linearize_builder(acc));
    }

}




#[impl_for_tuples(1,17)]
impl <'a, CS:'a+ConstraintSystem> Signal<'a, CS> for Tuple {

    for_tuples!( type Value = ( #( Tuple::Value ),* ); );

    fn get_value(&self) -> Option<Self::Value> {
        Some((for_tuples!( #( self.Tuple.get_value()?),* )))
    }

    fn switch(&self, bit: &CBool<'a, CS>, if_else: &Self) -> Self {
        (for_tuples!( #(self.Tuple.switch(bit, &if_else.Tuple) ),* ))
    }


    fn get_cs(&self) -> &'a CS {
        self.0.get_cs()
    }

    fn from_const(cs:&'a CS, value: &Self::Value) -> Self {
        (for_tuples!( #( Tuple::from_const(cs, &value.Tuple)),* ))
    }

    fn as_const(&self) -> Option<Self::Value> {
        Some((for_tuples!( #( self.Tuple.as_const()?),* )))
    }


    fn alloc(cs:&'a CS, value:Option<&Self::Value>) -> Self {
        match value {
            Some(value) => (for_tuples!( #( Tuple::alloc(cs, Some(&value.Tuple) )),* )),
            _ =>  (for_tuples!( #( Tuple::alloc(cs, None)),* ))
        }
    }

    fn assert_const(&self, value: &Self::Value) {
        for_tuples!( #(self.Tuple.assert_const(&value.Tuple); )* );
    }

    fn inputize(&self) {
        for_tuples!( #(self.Tuple.inputize(); )* );
    }
    
    fn assert_eq(&self, other: &Self) {
        for_tuples!( #(self.Tuple.assert_eq(&other.Tuple); )* );
    }

    fn is_eq(&self, other:&Self) -> CBool<'a, CS> {
        let mut acc = CNum::one(self.get_cs());
        for_tuples!( #(acc *= self.Tuple.is_eq(&other.Tuple).into_num(); )* );
        acc.into_bool()

    }


    fn linearize_builder(&self, acc: &mut Vec<CNum<'a, CS>>) {
        for_tuples!( #( self.Tuple.linearize_builder(acc); )* );
    }

}
