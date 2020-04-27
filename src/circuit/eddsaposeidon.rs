use ff::{PrimeField};

use crate::core::signal::{CNum};
use crate::core::abstractsignal::Signal;
use crate::core::cs::ConstraintSystem;
use crate::native::ecc::{JubJubParams};
use crate::native::poseidon::{PoseidonParams};
use crate::circuit::poseidon::{c_poseidon};
use crate::circuit::ecc::{CEdwardsPoint};
use crate::circuit::bitify::{c_into_bits_le_strict, c_into_bits_le};




pub fn c_eddsaposeidon_verify<'a, CS: ConstraintSystem, J:JubJubParams<CS::F>>(
    s: &CNum<'a, CS>,
    r: &CNum<'a, CS>,
    a: &CNum<'a, CS>,
    m: &CNum<'a, CS>,
    poseidon_params: &PoseidonParams<CS::F>,
    jubjub_params:&J
) -> CNum<'a, CS> {
    assert!(CS::F::NUM_BITS > J::Fs::NUM_BITS, "jubjub field should be lesser than snark field");
    let cs = s.cs;
    
    let p_a = CEdwardsPoint::subgroup_decompress(a, jubjub_params);
    let p_r = CEdwardsPoint::subgroup_decompress(r, jubjub_params);
    let h = c_poseidon(&[r.clone(), a.clone(), m.clone()], poseidon_params);
    let h_bits = c_into_bits_le_strict(&h);
    let ha = p_a.mul(&h_bits, jubjub_params);

    let s_bits = c_into_bits_le(&s, J::Fs::NUM_BITS as usize);
    let jubjub_generator = CEdwardsPoint::from_const(cs,jubjub_params.edwards_g().clone());
    let sb = jubjub_generator.mul(&s_bits, jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    (&ha_plus_r.x - &sb.x).is_zero()
}


#[cfg(test)]
mod eddsaposeidon_test {
    use super::*;
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};
    use crate::core::cs::TestCS;
    use crate::core::num::Num;
    use crate::core::abstractsignal::Signal;
    use crate::native::ecc::{JubJubBN256};
    use crate::native::eddsaposeidon::eddsaposeidon_sign;

    #[test]
    fn test_circuit_eddsaposeidon_verify() {
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(4, 8, 54);
        let jubjub_params = JubJubBN256::new();

        let sk = rng.gen();
        let m = rng.gen();
        let (s, r) = eddsaposeidon_sign(sk, m, &poseidon_params, &jubjub_params);
        let a = jubjub_params.edwards_g().mul(sk, &jubjub_params).into_xy().0;
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_s = CNum::alloc(cs, Some(s.into_other()));
        let signal_r = CNum::alloc(cs, Some(r));
        let signal_a = CNum::alloc(cs, Some(a));
        let signal_m = CNum::alloc(cs, Some(m));

        let mut n_constraints = cs.num_constraints();
        let res = c_eddsaposeidon_verify(&signal_s, &signal_r, &signal_a, &signal_m, &poseidon_params, &jubjub_params);
        n_constraints=cs.num_constraints()-n_constraints;
        
        res.assert_const(Num::one());
        println!("eddsaposeidon_verify constraints = {}", n_constraints);
        assert!(res.get_value().unwrap() == Num::one());
    }
}