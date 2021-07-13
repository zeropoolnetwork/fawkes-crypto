use crate::{
    constants::{PERSONALIZATION},
    ff_uint::{Num, NumRepr, PrimeField, Uint},
    native::{
        ecc::{EdwardsPoint, JubJubParams},
        poseidon::{poseidon, PoseidonParams},
    },
};

use byteorder::{ByteOrder, LittleEndian};
use blake2_rfc::blake2s::Blake2s;

fn hash_r<Fr: PrimeField, Fs: PrimeField>(sk: Num<Fs>, m: Num<Fr>) -> Num<Fs> {
    let mut h = Blake2s::with_params(32, &[], &[], PERSONALIZATION);

    sk.to_uint().as_inner().as_ref().iter().for_each(|x| h.update(&x.to_le_bytes()));
    m.to_uint().as_inner().as_ref().iter().for_each(|x| h.update(&x.to_le_bytes()));

    let res = h.finalize();
    let res_ref = res.as_ref();

    let mut n = NumRepr::<Fs::Inner>::ZERO;
    assert!(res_ref.len()*8 == Fs::Inner::NUM_WORDS * Fs::Inner::WORD_BITS);
    n.as_inner_mut().as_mut().iter_mut().enumerate().for_each(|(i, x)| {
        *x = LittleEndian::read_u64(&res_ref[i * 8..(i + 1) * 8]);
    });

    Num::from_uint_reduced(n)
}

fn hash_ram<F: PrimeField>(
    r: Num<F>,
    a: Num<F>,
    m: Num<F>,
    poseidon_params: &PoseidonParams<F>,
) -> Num<F> {
    poseidon(&[r, a, m], poseidon_params)
}

pub fn eddsaposeidon_sign<Fr: PrimeField, J: JubJubParams<Fr = Fr>>(
    sk: Num<J::Fs>,
    m: Num<Fr>,
    poseidon_params: &PoseidonParams<Fr>,
    jubjub_params: &J,
) -> (Num<J::Fs>, Num<Fr>) {
    let rho = hash_r(sk, m);
    let r_x = jubjub_params.edwards_g().mul(rho, jubjub_params).x;
    let a_x = jubjub_params.edwards_g().mul(sk, jubjub_params).x;
    let s = rho + hash_ram(r_x, a_x, m, poseidon_params).to_other_reduced() * sk;
    (s, r_x)
}

pub fn eddsaposeidon_verify<Fr: PrimeField, J: JubJubParams<Fr = Fr>>(
    s: Num<J::Fs>,
    r: Num<Fr>,
    a: Num<Fr>,
    m: Num<Fr>,
    poseidon_params: &PoseidonParams<Fr>,
    jubjub_params: &J,
) -> bool {
    let p_a = match EdwardsPoint::subgroup_decompress(a, jubjub_params) {
        Some(x) => x,
        _ => return false,
    };

    let p_r = match EdwardsPoint::subgroup_decompress(r, jubjub_params) {
        Some(x) => x,
        _ => return false,
    };

    let ha = p_a.mul(
        hash_ram(r, a, m, poseidon_params).to_other_reduced(),
        jubjub_params,
    );
    let sb = jubjub_params.edwards_g().mul(s, jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    sb == ha_plus_r
}
