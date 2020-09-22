use super::bool::SignalBool;

pub trait SignalNum : Sized {
    type Bool: SignalBool<Num=Self>;
}

