use crate::{
    constants::SEED_EDWARDS_G,
    engines::U256,
    ff_uint::{construct_primefield_params, Num, seedbox::{FromSeed, SeedboxChaCha20}},
    native::ecc::{EdwardsPoint, JubJubParams},
};

construct_primefield_params! {
    pub struct Fq(super::U256);

    impl PrimeFieldParams for Fq {
        type Inner = super::U256;
        const MODULUS: &'static str = "21888242871839275222246405745257275088696311157297823662689037894645226208583";
        const GENERATOR: &'static str = "2";
   }
}

construct_primefield_params! {
    pub struct Fr(super::U256);

    impl PrimeFieldParams for Fr {
        type Inner = super::U256;
        const MODULUS: &'static str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";
        const GENERATOR: &'static str = "7";
   }
}

construct_primefield_params! {
    pub struct Fs(super::U256);

    impl PrimeFieldParams for Fs {
        type Inner = super::U256;
        const MODULUS: &'static str = "2736030358979909402780800718157159386076813972158567259200215660948447373041";
        const GENERATOR: &'static str = "7";
   }
}


#[derive(Clone)]
pub struct JubJubBN256 {
    edwards_g: EdwardsPoint<Fr>,
    edwards_d: Num<Fr>,
    montgomery_a: Num<Fr>,
    montgomery_b: Num<Fr>,
    montgomery_u: Num<Fr>,
}

impl JubJubBN256 {
    pub fn new() -> Self {
        let edwards_d = -Num::from(168696) / Num::from(168700);

        let montgomery_a = Num::from(2) * (Num::ONE - edwards_d) / (Num::ONE + edwards_d);
        let montgomery_b = -Num::from(4) / (Num::ONE + edwards_d);

        // value of polynomial g(x)=(x^3+montgomery_a*x^2+x)/montgomery_b at x=montgomery_b (has no square root in Fr)
        let montgomery_u = Num::from(337401);

        let edwards_g = EdwardsPoint::from_scalar_raw(
        FromSeed::<SeedboxChaCha20>::from_seed(SEED_EDWARDS_G),
            montgomery_a,
            montgomery_b,
            montgomery_u,
        );
        Self {
            edwards_g,
            edwards_d,
            montgomery_a,
            montgomery_b,
            montgomery_u,
        }
    }
}

impl JubJubParams for JubJubBN256 {
    type Fr = Fr;
    type Fs = Fs;

    fn edwards_g(&self) -> &EdwardsPoint<Fr> {
        &self.edwards_g
    }

    fn edwards_d(&self) -> Num<Fr> {
        self.edwards_d
    }

    fn montgomery_a(&self) -> Num<Fr> {
        self.montgomery_a
    }

    fn montgomery_b(&self) -> Num<Fr> {
        self.montgomery_b
    }

    fn montgomery_u(&self) -> Num<Fr> {
        self.montgomery_u
    }
}
