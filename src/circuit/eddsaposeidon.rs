use ff::{PrimeField};

use crate::core::signal::{Signal};
use crate::core::cs::ConstraintSystem;
use crate::native::ecc::{JubJubParams};
use crate::native::poseidon::{PoseidonParams};
use crate::circuit::poseidon::{poseidon};
use crate::circuit::ecc::{EdwardsPoint};
use crate::circuit::bitify::{into_bits_le_strict, into_bits_le};




pub fn eddsaposeidon_verify<'a, CS: ConstraintSystem, J:JubJubParams<CS::F>>(
    s: &Signal<'a, CS>,
    r: &Signal<'a, CS>,
    a: &Signal<'a, CS>,
    m: &Signal<'a, CS>,
    poseidon_params: &PoseidonParams<CS::F>,
    jubjub_params:&J
) -> Signal<'a, CS> {
    assert!(CS::F::NUM_BITS > J::Fs::NUM_BITS, "jubjub field should be lesser than snark field");
    let cs = s.cs;
    
    let p_a = EdwardsPoint::subgroup_decompress(a, jubjub_params);
    let p_r = EdwardsPoint::subgroup_decompress(r, jubjub_params);
    let h = poseidon(&[r.clone(), a.clone(), m.clone()], poseidon_params);
    let h_bits = into_bits_le_strict(&h);
    let ha = p_a.multiply(&h_bits, jubjub_params);

    let s_bits = into_bits_le(&s, J::Fs::NUM_BITS as usize);
    let jubjub_generator = EdwardsPoint::from_const(cs,jubjub_params.edwards_g().clone());
    let sb = jubjub_generator.multiply(&s_bits, jubjub_params);
    let ha_plus_r = ha.add(&p_r, jubjub_params);

    (&ha_plus_r.x - &sb.x).is_zero()
}


#[cfg(test)]
mod eddsaposeidon_test {
    use super::*;
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};
    use crate::native::ecc::{JubJubBN256};
    use crate::core::cs::TestCS;
    use crate::core::num::Num;

    #[test]
    fn test_circuit_eddsaposeidon_verify() {
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(4, 8, 54);
        let jubjub_params = JubJubBN256::new();

        let sk = rng.gen();
        let m = rng.gen();
        let (s, r) = crate::native::eddsaposeidon::eddsaposeidon_sign(sk, m, &poseidon_params, &jubjub_params);
        let a = jubjub_params.edwards_g().mul(sk, &jubjub_params).into_xy().0;
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_s = Signal::alloc(cs, Some(s.into_other()));
        let signal_r = Signal::alloc(cs, Some(r));
        let signal_a = Signal::alloc(cs, Some(a));
        let signal_m = Signal::alloc(cs, Some(m));

        let mut n_constraints = cs.num_constraints();
        let res = eddsaposeidon_verify(&signal_s, &signal_r, &signal_a, &signal_m, &poseidon_params, &jubjub_params);
        n_constraints=cs.num_constraints()-n_constraints;
        
        res.assert_const(Num::one());
        println!("eddsaposeidon_verify constraints = {}", n_constraints);
        assert!(res.get_value().unwrap() == Num::one());
    }
}