use ff_uint::construct_primefield_params;
use crate::engines::U256;


construct_primefield_params! {
    pub struct _Fq(super::U256);

    impl PrimeFieldParams for _Fq {
        type Inner = super::U256;
        const MODULUS: &'static str = "21888242871839275222246405745257275088696311157297823662689037894645226208583";
        const GENERATOR: &'static str = "2";
   }
}


construct_primefield_params! {
    pub struct _Fr(super::U256);

    impl PrimeFieldParams for _Fr {
        type Inner = super::U256;
        const MODULUS: &'static str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";
        const GENERATOR: &'static str = "7";
   }
}


construct_primefield_params! {
    pub struct _Fs(super::U256);

    impl PrimeFieldParams for _Fs {
        type Inner = super::U256;
        const MODULUS: &'static str = "2736030358979909402780800718157159386076813972158567259200215660948447373041";
        const GENERATOR: &'static str = "7";
   }
}

pub type Fq = _Fq;
pub type Fr = _Fr;
pub type Fs = _Fs;