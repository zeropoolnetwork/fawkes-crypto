use crate::core::field::{PrimeField, PrimeFieldRepr, AbstractField};
use crate::native::{ecc::{EdwardsPoint, JubJubParams}, num::Num};
use crate::constants::SEED_EDWARDS_G;

pub use bellman::pairing::bls12_381::Fr;


#[derive(PrimeField)]
#[PrimeFieldModulus = "6554484396890773809930967563523245729705921265872317281365359162392183254199"]
#[PrimeFieldGenerator = "7"]
pub struct Fs(FsRepr);

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
        let edwards_d = -num!(10240)/num!(10241);

        let montgomery_a = num!(2)*(Num::one()-edwards_d)/(Num::one()+edwards_d);
        let montgomery_b = -num!(4)/(Num::one()+edwards_d);
        
        // value of montgomery polynomial for x=montgomery_b (has no square root in Fr)
        let montgomery_u= num!(81929);

        let edwards_g = EdwardsPoint::from_scalar_raw(Num::from_seed(SEED_EDWARDS_G), montgomery_a, montgomery_b, montgomery_u);
        Self {
            edwards_g,
            edwards_d,
            montgomery_a,
            montgomery_b,
            montgomery_u
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
