use ff::{
    PrimeField, SqrtField
};

use blake2_rfc::blake2s::Blake2s;
use crate::core::num::Num;


use crate::native::ecc::{EdwardsPoint, JubJubParams};
use crate::native::poseidon::{PoseidonParams, poseidon};


fn hash_r<Fr:PrimeField, Fs:PrimeField>(
    sk: Num<Fs>,
    m: Num<Fr>,
) -> Num<Fs> {
    let mut h = Blake2s::with_params(32, &[], &[], b"faw_eddR");
    h.update(sk.into_binary_be().as_ref());
    h.update(m.into_binary_be().as_ref());
    Num::<Fs>::from_binary_be(h.finalize().as_ref())
}

fn hash_ram<F: PrimeField>(
    r:Num<F>,
    a:Num<F>,
    m:Num<F>,
    poseidon_params: &PoseidonParams<F>
) -> Num<F> {
    poseidon(&[r, a, m], poseidon_params)
}

pub fn eddsaposeidon_sign<Fr:PrimeField, J:JubJubParams<Fr>>(
    sk: Num<J::Fs>,
    m: Num<Fr>,
    poseidon_params: &PoseidonParams<Fr>,
    jubjub_params:&J
) -> (Num<J::Fs>, Num<Fr>) {
    let rho = hash_r(sk, m);
    let (r_x, _) = jubjub_params.edwards_g8().mul(rho, jubjub_params).into_xy();
    let (a_x, _) = jubjub_params.edwards_g8().mul(sk, jubjub_params).into_xy();
    let s = rho + hash_ram(r_x, a_x, m, poseidon_params).into_other()*sk;
    (s, r_x)
}


pub fn eddsaposeidon_verify<Fr:PrimeField+SqrtField, J:JubJubParams<Fr>>(
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
    let sb = jubjub_params.edwards_g8().mul(s, jubjub_params);
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
        let a = jubjub_params.edwards_g8().mul(sk, &jubjub_params).into_xy().0;
        assert!(eddsaposeidon_verify(s, r, a, m, &poseidon_params, &jubjub_params), "signature should be valid");
    
    }



}