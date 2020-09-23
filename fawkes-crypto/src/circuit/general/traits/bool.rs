use super::num::SignalNum;
use std::ops::{Not, BitAndAssign, BitOrAssign, BitXorAssign, BitAnd, BitOr, BitXor};
pub trait SignalBool : Sized + Not + BitAndAssign<Self> + for<'a> BitAndAssign<&'a Self> {
    type Num: SignalNum<Bool=Self>;

    fn new_unchecked(n:&Self::Num) -> Self;

    fn new(n: &Self::Num) -> Self;

    fn to_num(&self) -> Self::Num;
}

pub trait Hello: SignalBool {}