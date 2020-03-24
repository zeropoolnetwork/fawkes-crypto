use bellman::pairing::{
    Engine
};

use ff::{
    PrimeField
};

use blake2_rfc::blake2s::Blake2s;
use crate::wrappedmath::Wrap;


use crate::ecc::{EdwardsPoint, JubJubParams};
use crate::poseidon::{PoseidonParams, poseidon};


fn hash_r<Fr:PrimeField, Fs:PrimeField>(
    sk: Wrap<Fs>,
    m: Wrap<Fr>,
) -> Wrap<Fs> {
    let mut h = Blake2s::with_params(32, &[], &[], b"faw_eddR");
    h.update(sk.into_binary_be().as_ref());
    h.update(m.into_binary_be().as_ref());
    Wrap::<Fs>::from_binary_be(h.finalize().as_ref())
}

fn hash_ram<F: PrimeField>(
    r:Wrap<F>,
    a:Wrap<F>,
    m:Wrap<F>,
    poseidon_params: &PoseidonParams<F>
) -> Wrap<F> {
    poseidon(&[r, a, m], poseidon_params)
}

pub fn eddsaposeidon_sign<E: Engine, J:JubJubParams<E>>(
    sk: Wrap<J::Fs>,
    m: Wrap<E::Fr>,
    poseidon_params: &PoseidonParams<E::Fr>,
    jubjub_params:&J
) -> (Wrap<J::Fs>, Wrap<E::Fr>) {
    let rho = hash_r(sk, m);
    let (r_x, _) = jubjub_params.edwards_g8().mul(rho.into_repr(), jubjub_params).into_xy();
    let (a_x, _) = jubjub_params.edwards_g8().mul(sk.into_repr(), jubjub_params).into_xy();
    let s = rho + Wrap::from_other(hash_ram(r_x, a_x, m, poseidon_params))*sk;
    (s, r_x)
}


pub fn eddsaposeidon_verify<E: Engine, J:JubJubParams<E>>(
    s: Wrap<J::Fs>,
    r: Wrap<E::Fr>,
    a: Wrap<E::Fr>,
    m: Wrap<E::Fr>,
    poseidon_params: &PoseidonParams<E::Fr>,
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

    let ha = p_a.mul(hash_ram(r, a, m, poseidon_params).into_repr(), jubjub_params);
    let sb = jubjub_params.edwards_g8().mul(s.into_repr(), jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    sb == ha_plus_r
}

#[cfg(test)]
mod eddsaposeidon_test {
    use super::*;

    use rand::{Rng, thread_rng};
    use bellman::pairing::bn256::{Fr};

    use crate::ecc::{JubJubBN256};




    #[test]
    fn eddsaposeidon() {
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(4, 8, 54);
        let jubjub_params = JubJubBN256::new();

        let sk = rng.gen();
        let m = rng.gen();
        let (s, r) = eddsaposeidon_sign(sk, m, &poseidon_params, &jubjub_params);
        let a = jubjub_params.edwards_g8().mul(sk.into_repr(), &jubjub_params).into_xy().0;
        assert!(eddsaposeidon_verify(s, r, a, m, &poseidon_params, &jubjub_params), "signature should be valid")
    
    }



}