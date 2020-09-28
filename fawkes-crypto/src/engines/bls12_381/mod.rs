use ff_uint::construct_primefield_params;
use crate::engines::{U256, U384};


construct_primefield_params! {
    pub struct _Fq(super::U384);

    impl PrimeFieldParams for _Fq {
        type Inner = super::U384;
        const MODULUS: &'static str = "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787";
        const GENERATOR: &'static str = "2";
   }
}


construct_primefield_params! {
    pub struct _Fr(super::U256);

    impl PrimeFieldParams for _Fr {
        type Inner = super::U256;
        const MODULUS: &'static str = "52435875175126190479447740508185965837690552500527637822603658699938581184513";
        const GENERATOR: &'static str = "7";
   }
}


construct_primefield_params! {
    pub struct _Fs(super::U256);

    impl PrimeFieldParams for _Fs {
        type Inner = super::U256;
        const MODULUS: &'static str = "6554484396890773809930967563523245729705921265872317281365359162392183254199";
        const GENERATOR: &'static str = "7";
   }
}

pub type Fq = _Fq;
pub type Fr = _Fr;
pub type Fs = _Fs;
