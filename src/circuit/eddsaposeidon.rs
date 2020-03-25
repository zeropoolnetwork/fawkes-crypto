use bellman::{
    SynthesisError,
    ConstraintSystem
};

use bellman::pairing::{
    Engine
};

use ff::{PrimeField};

use super::signal::{Signal};
use crate::ecc::{JubJubParams};
use crate::poseidon::{PoseidonParams};
use crate::circuit::poseidon::{poseidon};
use crate::circuit::ecc::{EdwardsPoint};
use crate::circuit::bitify::{into_bits_le_strict, into_bits_le};




pub fn eddsaposeidon_verify<E: Engine, J:JubJubParams<E>, CS:ConstraintSystem<E>>(
    mut cs:CS,
    s: &Signal<E>,
    r: &Signal<E>,
    a: &Signal<E>,
    m: &Signal<E>,
    poseidon_params: &PoseidonParams<E::Fr>,
    jubjub_params:&J
) -> Result<Signal<E>, SynthesisError> {
    assert!(E::Fr::NUM_BITS > J::Fs::NUM_BITS, "jubjub field should be lesser than snark field");
    
    let p_a = EdwardsPoint::subgroup_decompress(cs.namespace(|| "decompress A"), a, jubjub_params)?;
    let p_r = EdwardsPoint::subgroup_decompress(cs.namespace(|| "decompress R"), r, jubjub_params)?;
    let h = poseidon(cs.namespace(|| "compute H(R,A,M)"), &[r.clone(), a.clone(), m.clone()], poseidon_params)?;
    let h_bits = into_bits_le_strict(cs.namespace(|| "bitify h"), &h)?;
    let ha = p_a.multiply(cs.namespace(|| "compute h*A"), &h_bits, jubjub_params)?;

    let s_bits = into_bits_le(cs.namespace(|| "bitify s"), &s, J::Fs::NUM_BITS as usize)?;
    let jubjub_generator = EdwardsPoint::constant(jubjub_params.edwards_g8().clone());
    let sb = jubjub_generator.multiply(cs.namespace(|| "multiply s*B"), &s_bits, jubjub_params)?;
    let ha_plus_r = ha.add(cs.namespace(|| "compute hA+R"), &p_r, jubjub_params)?;

    (&ha_plus_r.x - &sb.x).is_zero(cs.namespace(|| "check sB == hA+R"))
}


#[cfg(test)]
mod eddsaposeidon_test {
    use super::*;
    use sapling_crypto::circuit::test::TestConstraintSystem;
    use bellman::pairing::bn256::{Bn256, Fr};
    use rand::{Rng, thread_rng};
    use crate::ecc::{JubJubBN256};
    use crate::wrappedmath::Wrap;

    #[test]
    fn test_circuit_eddsaposeidon_verify() {
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(4, 8, 54);
        let jubjub_params = JubJubBN256::new();

        let sk = rng.gen();
        let m = rng.gen();
        let (s, r) = crate::eddsaposeidon::eddsaposeidon_sign(sk, m, &poseidon_params, &jubjub_params);
        let a = jubjub_params.edwards_g8().mul(sk.into_repr(), &jubjub_params).into_xy().0;
        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_s = Signal::alloc(cs.namespace(||"s"), Some(Wrap::from_other(s))).unwrap();
        let signal_r = Signal::alloc(cs.namespace(||"r"), Some(r)).unwrap();
        let signal_a = Signal::alloc(cs.namespace(||"a"), Some(a)).unwrap();
        let signal_m = Signal::alloc(cs.namespace(||"m"), Some(m)).unwrap();

        let mut n_constraints = cs.num_constraints();
        let res = eddsaposeidon_verify(cs.namespace(||"verify"), &signal_s, &signal_r, &signal_a, &signal_m, &poseidon_params, &jubjub_params).unwrap();
        n_constraints=cs.num_constraints()-n_constraints;
        
        res.assert_constant(cs.namespace(|| "check res"), Wrap::one()).unwrap();

        println!("eddsaposeidon_verify constraints = {}", n_constraints);

        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }

        assert!(res.get_value().unwrap() == Wrap::one());
    }
}