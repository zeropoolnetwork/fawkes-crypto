use super::bool::SignalBool;

pub trait SignalNum : Sized {
    type Bool: SignalBool<Num=Self>;

    fn assert_zero(&self);
    fn assert_nonzero(&self);
    fn is_zero(&self) -> Self::Bool;
    fn assert_bit(&self);
    fn to_bool(&self) -> Self::Bool;
    fn to_bool_unchecked(&self) -> Self::Bool;
    fn from_bool(b:Self::Bool) -> Self;
    fn inv(&self) -> Self;
}

