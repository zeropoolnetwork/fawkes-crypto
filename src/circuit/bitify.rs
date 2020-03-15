use bellman_ce::pairing::{
    Engine,
};

use bellman_ce::pairing::ff::{
    Field,
    PrimeField,
    PrimeFieldRepr,
    BitIterator
};

use bellman_ce::{
    SynthesisError,
    ConstraintSystem,
    LinearCombination,
    Variable,
    Index
};


use std::ops::{Add, Sub, Mul};
use std::collections::HashMap;

use super::Assignment;
use super::signal::Signal;



fn fr_into_bits<F:PrimeField>(value:F) -> Vec<bool>{
    let value_repr = value.into_repr();
    let value_repr_ref = value_repr.as_ref();
    let nlimbs = value_repr_ref.len();

    let mut res = Vec::<bool>::new();
    for limb in 0..nlimbs {
        for offset in 0..64 {
            res.push(value_repr_ref[limb] >> offset & 1 == 1)
        }
    }
    res
    
}

fn bool2fr<F:PrimeField>(value:bool, f:F) -> F {
    if value {
        f
    } else {
        F::zero()
    }
}

pub fn into_bits_le<E:Engine, CS:ConstraintSystem<E>>(
    signal:&Signal<E>,
    mut cs: CS,
    limit: usize
) -> Result<Vec<Signal<E>>, SynthesisError>
    where CS: ConstraintSystem<E>
{

    match signal {
        Signal::Variable(_, _) => {
            let value = signal.get_value();
            let mut remained_signal = signal.clone();
            let mut k = E::Fr::one();
            let mut bits = Vec::<Signal<E>>::new();
            let value_bits = match value {
                Some(v) => fr_into_bits(v).into_iter().map(|e| Some(e)).collect::<Vec<_>>(),
                None => vec![None; E::Fr::NUM_BITS as usize]
            };
            
            value.map(|v|fr_into_bits(v));
            
            for i in 0..limit-1 {
                let s = Signal::alloc(cs.namespace(|| format!("alloc bit {}", i)), || value_bits[i].map(|b| bool2fr(b, E::Fr::one())).grab())?;
                s.assert_bit(cs.namespace(|| format!("assert bit {}", i)));
                remained_signal = remained_signal - &(k * &s);
                bits.push(s);
                k.double();
            }
            let remained_signal = remained_signal.normalize();
            remained_signal.assert_bit(cs.namespace(|| format!("assert last bit {}", limit-1)));
            bits.push(remained_signal);
            Ok(bits)
        },
        Signal::Constant(value) => {
            let mut bits = Vec::<Signal<E>>::new();
            let mut k = E::Fr::one();
            let mut remained_value = value.clone();
            let value_bits = fr_into_bits(*value);
            for i in 0..limit {
                let bit = bool2fr(value_bits[i], k.clone()); 
                remained_value.sub_assign(&bit);
                bits.push(Signal::Constant(bit));
                k.double();
            }
            if remained_value.is_zero() {
                Ok(bits)
            } else {
                Err(SynthesisError::Unsatisfiable)
            }
        }
    }


}