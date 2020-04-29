use crate::circuit::num::{CNum};
use crate::core::cs::ConstraintSystem;
use crate::core::signal::Signal;
use crate::native::num::Num;

#[derive(Debug, Clone)]
pub struct CBool<'a, CS:ConstraintSystem>(pub CNum<'a, CS>);

impl<'a, CS:ConstraintSystem> Signal<'a, CS> for CBool<'a, CS> {
    type Value = bool;

    fn get_cs(&self) -> &'a CS {
        self.0.get_cs()
    }

    fn from_const(cs:&'a CS, value: &Self::Value) -> Self {
        CBool(CNum::from_const(cs, &Num::from(value.clone())))
    }
    
    fn get_value(&self) -> Option<Self::Value> {
        self.0.get_value().map(|n| !n.is_zero())
    }

    fn alloc(cs:&'a CS, value:Option<&Self::Value>) -> Self {
        CBool(CNum::alloc(cs, value.map(|v| Num::from(v.clone())).as_ref()))
    }

    fn switch(&self, bit: &CBool<'a, CS>, if_else: &Self) -> Self {
        CBool(self.0.switch(bit, &if_else.0))
    }
} 


impl <'a, CS:ConstraintSystem> CBool<'a, CS> {
    
    #[inline]
    pub fn assert(&self) {
        self.0.assert_bit();
    }

    #[inline]
    pub fn c_true(cs:&'a CS) -> Self {
        Self::from_const(cs, &true)
    }

    #[inline]
    pub fn c_false(cs:&'a CS) -> Self {
        Self::from_const(cs, &false)
    }
}