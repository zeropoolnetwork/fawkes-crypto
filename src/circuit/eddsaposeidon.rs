use crate::{
    circuit::bitify::{c_into_bits_le, c_into_bits_le_strict},
    circuit::bool::CBool,
    circuit::ecc::CEdwardsPoint,
    circuit::num::CNum,
    circuit::poseidon::c_poseidon_with_salt,
    constants::SEED_EDDSA_POSEIDON,
    core::cs::ConstraintSystem,
    core::field::PrimeField,
    core::signal::Signal,
    native::ecc::JubJubParams,
    native::poseidon::PoseidonParams,
};

pub fn c_eddsaposeidon_verify<'a, CS: ConstraintSystem, J: JubJubParams<Fr = CS::F>>(
    s: &CNum<'a, CS>,
    r: &CNum<'a, CS>,
    a: &CNum<'a, CS>,
    m: &CNum<'a, CS>,
    poseidon_params: &PoseidonParams<CS::F>,
    jubjub_params: &J,
) -> CBool<'a, CS> {
    assert!(
        CS::F::NUM_BITS > J::Fs::NUM_BITS,
        "jubjub field should be lesser than snark field"
    );
    let cs = s.cs;

    let p_a = CEdwardsPoint::subgroup_decompress(a, jubjub_params);
    let p_r = CEdwardsPoint::subgroup_decompress(r, jubjub_params);
    let h = c_poseidon_with_salt(
        &[r.clone(), a.clone(), m.clone()],
        SEED_EDDSA_POSEIDON,
        poseidon_params,
    );
    let h_bits = c_into_bits_le_strict(&h);
    let ha = p_a.mul(&h_bits, jubjub_params);

    let s_bits = c_into_bits_le(&s, J::Fs::NUM_BITS as usize);
    let jubjub_generator = CEdwardsPoint::from_const(cs, jubjub_params.edwards_g());
    let sb = jubjub_generator.mul(&s_bits, jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    (&ha_plus_r.x - &sb.x).is_zero()
}

#[cfg(test)]
mod eddsaposeidon_test {
    use super::*;
    use crate::core::cs::TestCS;
    use crate::native::bn256::{Fr, JubJubBN256};
    use crate::native::eddsaposeidon::eddsaposeidon_sign;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_circuit_eddsaposeidon_verify() {
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(4, 8, 54);
        let jubjub_params = JubJubBN256::new();

        let sk = rng.gen();
        let m = rng.gen();
        let (s, r) = eddsaposeidon_sign(sk, m, &poseidon_params, &jubjub_params);
        let a = jubjub_params.edwards_g().mul(sk, &jubjub_params).x;

        let ref mut cs = TestCS::<Fr>::new();
        let signal_s = CNum::alloc(cs, Some(&s.into_other()));
        let signal_r = CNum::alloc(cs, Some(&r));
        let signal_a = CNum::alloc(cs, Some(&a));
        let signal_m = CNum::alloc(cs, Some(&m));

        let mut n_constraints = cs.num_constraints();
        let res = c_eddsaposeidon_verify(
            &signal_s,
            &signal_r,
            &signal_a,
            &signal_m,
            &poseidon_params,
            &jubjub_params,
        );
        n_constraints = cs.num_constraints() - n_constraints;

        res.assert_true();
        println!("eddsaposeidon_verify constraints = {}", n_constraints);
        assert!(res.get_value().unwrap());
    }
}
