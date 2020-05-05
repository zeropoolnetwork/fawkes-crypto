use crate::core::field::Field;

use blake2_rfc::blake2s::Blake2s;
use crate::native::num::Num;


use crate::native::ecc::{EdwardsPoint, JubJubParams};
use crate::native::poseidon::{PoseidonParams, poseidon_with_salt};
use crate::constants::{PERSONALIZATION, SEED_EDDSA_POSEIDON};

fn hash_r<Fr:Field, Fs:Field>(
    sk: Num<Fs>,
    m: Num<Fr>,
) -> Num<Fs> {
    let mut h = Blake2s::with_params(32, &[], &[], PERSONALIZATION);
    h.update(sk.into_binary_be().as_ref());
    h.update(m.into_binary_be().as_ref());
    Num::<Fs>::from_binary_be(h.finalize().as_ref())
}

fn hash_ram<F: Field>(
    r:Num<F>,
    a:Num<F>,
    m:Num<F>,
    poseidon_params: &PoseidonParams<F>
) -> Num<F> {
    poseidon_with_salt(&[r, a, m], SEED_EDDSA_POSEIDON, poseidon_params)
}

pub fn eddsaposeidon_sign<Fr:Field, J:JubJubParams<Fr>>(
    sk: Num<J::Fs>,
    m: Num<Fr>,
    poseidon_params: &PoseidonParams<Fr>,
    jubjub_params:&J
) -> (Num<J::Fs>, Num<Fr>) {
    let rho = hash_r(sk, m);
    let r_x = jubjub_params.edwards_g().mul(rho, jubjub_params).x;
    let a_x = jubjub_params.edwards_g().mul(sk, jubjub_params).x;
    let s = rho + hash_ram(r_x, a_x, m, poseidon_params).into_other()*sk;
    (s, r_x)
}


pub fn eddsaposeidon_verify<Fr:Field, J:JubJubParams<Fr>>(
    s: Num<J::Fs>,
    r: Num<Fr>,
    a: Num<Fr>,
    m: Num<Fr>,
    poseidon_params: &PoseidonParams<Fr>,
    jubjub_params:&J
) -> bool {
    let p_a = match EdwardsPoint::subgroup_decompress(a, jubjub_params) {
        Some(x) => x,
        _ => return false
    };

    let p_r = match EdwardsPoint::subgroup_decompress(r, jubjub_params) {
        Some(x) => x,
        _ => return false
    };

    let ha = p_a.mul(hash_ram(r, a, m, poseidon_params).into_other(), jubjub_params);
    let sb = jubjub_params.edwards_g().mul(s, jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    sb == ha_plus_r
}

#[cfg(test)]
mod eddsaposeidon_test {
    use super::*;

    use rand::{Rng, thread_rng};
    use bellman::pairing::bn256::{Fr};

    use crate::native::ecc::{JubJubBN256};

    #[test]
    fn test_eddsaposeidon() {
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(4, 8, 54);
        let jubjub_params = JubJubBN256::new();

        let sk = rng.gen();
        let m = rng.gen();
        let (s, r) = eddsaposeidon_sign(sk, m, &poseidon_params, &jubjub_params);
        let a = jubjub_params.edwards_g().mul(sk, &jubjub_params).x;
        assert!(eddsaposeidon_verify(s, r, a, m, &poseidon_params, &jubjub_params), "signature should be valid");
    
    }



}