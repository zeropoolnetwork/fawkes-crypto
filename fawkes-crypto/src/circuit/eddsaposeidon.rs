use crate::{
    circuit::{
        bitify::{c_into_bits_le, c_into_bits_le_strict},
        bool::CBool,
        ecc::CEdwardsPoint,
        num::CNum,
        poseidon::c_poseidon_with_salt,
    },
    constants::SEED_EDDSA_POSEIDON,
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
    native::{ecc::JubJubParams, poseidon::PoseidonParams},
};

pub fn c_eddsaposeidon_verify<Fr: PrimeField, J: JubJubParams<Fr = Fr>>(
    s: &CNum<Fr>,
    r: &CNum<Fr>,
    a: &CNum<Fr>,
    m: &CNum<Fr>,
    poseidon_params: &PoseidonParams<Fr>,
    jubjub_params: &J,
) -> CBool<Fr> {
    assert!(
        Num::<Fr>::MODULUS_BITS > Num::<J::Fs>::MODULUS_BITS,
        "jubjub field should be lesser than snark field"
    );
    let cs = s.get_cs();

    let p_a = CEdwardsPoint::subgroup_decompress(a, jubjub_params);
    let p_r = CEdwardsPoint::subgroup_decompress(r, jubjub_params);
    let h = c_poseidon_with_salt(
        &[r.clone(), a.clone(), m.clone()],
        SEED_EDDSA_POSEIDON,
        poseidon_params,
    );
    let h_bits = c_into_bits_le_strict(&h);
    let ha = p_a.mul(&h_bits, jubjub_params);

    let s_bits = c_into_bits_le(&s, Num::<J::Fs>::MODULUS_BITS as usize);
    let jubjub_generator = CEdwardsPoint::from_const(cs, jubjub_params.edwards_g());
    let sb = jubjub_generator.mul(&s_bits, jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    (&ha_plus_r.x - &sb.x).is_zero()
}
