use crate::{
    core::field::{AbstractField, PrimeField, PrimeFieldRepr},
    constants::SEED_EDWARDS_G,
    native::{
        ecc::{EdwardsPoint, JubJubParams},
        num::Num,
    }
};

pub use bellman::pairing::bn256::Fr;

#[derive(PrimeField)]
#[PrimeFieldModulus = "2736030358979909402780800718157159386076813972158567259200215660948447373041"]
#[PrimeFieldGenerator = "7"]
pub struct Fs(FsRepr);

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
        let edwards_d = -num!(168696) / num!(168700);

        let montgomery_a = num!(2) * (Num::one() - edwards_d) / (Num::one() + edwards_d);
        let montgomery_b = -num!(4) / (Num::one() + edwards_d);

        // value of montgomery polynomial for x=montgomery_b (has no square root in Fr)
        let montgomery_u = num!(337401);

        let edwards_g = EdwardsPoint::from_scalar_raw(
            Num::from_seed(SEED_EDWARDS_G),
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
