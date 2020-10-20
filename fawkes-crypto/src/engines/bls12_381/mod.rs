use crate::{
    constants::SEED_EDWARDS_G,
    engines::{U256, U384},
    ff_uint::{construct_primefield_params, Num, seedbox::{FromSeed, SeedboxBlake2}},
    native::ecc::{EdwardsPoint, JubJubParams},
};

construct_primefield_params! {
    pub struct Fq(super::U384);

    impl PrimeFieldParams for Fq {
        type Inner = super::U384;
        const MODULUS: &'static str = "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787";
        const GENERATOR: &'static str = "2";
   }
}

construct_primefield_params! {
    pub struct Fr(super::U256);

    impl PrimeFieldParams for Fr {
        type Inner = super::U256;
        const MODULUS: &'static str = "52435875175126190479447740508185965837690552500527637822603658699938581184513";
        const GENERATOR: &'static str = "7";
   }
}

construct_primefield_params! {
    pub struct Fs(super::U256);

    impl PrimeFieldParams for Fs {
        type Inner = super::U256;
        const MODULUS: &'static str = "6554484396890773809930967563523245729705921265872317281365359162392183254199";
        const GENERATOR: &'static str = "7";
   }
}


#[derive(Clone)]
pub struct JubJubBLS12_381 {
    edwards_g: EdwardsPoint<Fr>,
    edwards_d: Num<Fr>,
    montgomery_a: Num<Fr>,
    montgomery_b: Num<Fr>,
    montgomery_u: Num<Fr>,
}

impl JubJubBLS12_381 {
    pub fn new() -> Self {
        let edwards_d = -Num::from(10240) / Num::from(10241);

        let montgomery_a = Num::from(2) * (Num::ONE - edwards_d) / (Num::ONE + edwards_d);
        let montgomery_b = -Num::from(4) / (Num::ONE + edwards_d);

        // value of montgomery polynomial for x=montgomery_b (has no square root in Fr)
        let montgomery_u = Num::from(81929);

        let edwards_g = EdwardsPoint::from_scalar_raw(
            FromSeed::<SeedboxBlake2>::from_seed(SEED_EDWARDS_G),
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

impl JubJubParams for JubJubBLS12_381 {
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
