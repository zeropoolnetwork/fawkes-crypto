use crate::{
    circuit::{
        bitify::{c_into_bits_le, c_into_bits_le_strict},
        bool::CBool,
        ecc::CEdwardsPoint,
        num::CNum,
        poseidon::c_poseidon,
        cs::CS,
    },
    core::signal::Signal,
    ff_uint::Num,
    native::{ecc::JubJubParams, poseidon::PoseidonParams},
};

pub fn c_eddsaposeidon_verify<C: CS, J: JubJubParams<Fr = C::Fr>>(
    s: &CNum<C>,
    r: &CNum<C>,
    a: &CNum<C>,
    m: &CNum<C>,
    poseidon_params: &PoseidonParams<C::Fr>,
    jubjub_params: &J,
) -> CBool<C> {
    assert!(
        Num::<C::Fr>::MODULUS_BITS > Num::<J::Fs>::MODULUS_BITS,
        "jubjub field should be lesser than snark field"
    );
    let cs = s.get_cs();

    let p_a = CEdwardsPoint::subgroup_decompress(a, jubjub_params);
    let p_r = CEdwardsPoint::subgroup_decompress(r, jubjub_params);
    let h = c_poseidon(
        &[r.clone(), a.clone(), m.clone()],
        poseidon_params,
    );
    let h_bits = c_into_bits_le_strict(&h);
    let ha = p_a.mul(&h_bits, jubjub_params);

    let s_bits = c_into_bits_le(s, Num::<J::Fs>::MODULUS_BITS as usize);
    let jubjub_generator = CEdwardsPoint::from_const(cs, jubjub_params.edwards_g());
    let sb = jubjub_generator.mul(&s_bits, jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    (&ha_plus_r.x - &sb.x).is_zero()
}
