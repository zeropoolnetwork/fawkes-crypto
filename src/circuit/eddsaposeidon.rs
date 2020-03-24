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