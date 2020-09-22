use super::num::SignalNum;

pub trait SignalBool : Sized {
    type Num: SignalNum<Bool=Self>;

}