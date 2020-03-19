use bellman_ce::pairing::{
    Engine
};

use bellman_ce::pairing::ff::{
    Field,
    PrimeField
};

use bellman_ce::{
    SynthesisError,
    ConstraintSystem
};



use super::Assignment;
use super::signal::Signal;
use crate::bititerator::BitIteratorLE;


fn bool2fr<F:PrimeField>(value:bool, f:F) -> F {
    if value {
        f
    } else {
        F::zero()
    }
}

pub fn into_bits_le<E:Engine, CS:ConstraintSystem<E>>(
    mut cs: CS,
    signal:&Signal<E>,
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
                Some(v) => BitIteratorLE::new(v.into_repr()).map(|e| Some(e)).collect::<Vec<_>>(),
                None => vec![None; E::Fr::NUM_BITS as usize]
            };
            
            for i in 0..limit-1 {
                let s = Signal::alloc(cs.namespace(|| format!("alloc bit {}", i)), || value_bits[i].map(|b| bool2fr(b, E::Fr::one())).grab())?;
                s.assert_bit(cs.namespace(|| format!("assert bit {}", i)))?;
                remained_signal = remained_signal - &(k * &s);
                bits.push(s);
                k.double();
            }
            let remained_signal = remained_signal.normalize();
            remained_signal.assert_bit(cs.namespace(|| format!("assert last bit {}", limit-1)))?;
            bits.push(remained_signal);
            Ok(bits)
        },
        Signal::Constant(value) => {
            let mut bits = Vec::<Signal<E>>::new();
            let mut k = E::Fr::one();
            let mut remained_value = value.clone();
            let value_bits = BitIteratorLE::new(value.into_repr()).collect::<Vec<_>>();
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

// return 1 if ct > signal
pub fn comp_constant<E:Engine, CS:ConstraintSystem<E>>(
    mut cs: CS,
    signal:&[Signal<E>],
    ct: &E::Fr
) -> Result<Signal<E>, SynthesisError> {
    let siglen = signal.len();
    let nsteps = (siglen >> 1) + (siglen & 1);
    let sig_zero = if siglen & 1 == 1 {vec![Signal::<E>::zero()] } else {vec![]};

    let mut sig_bits = signal.iter().chain(sig_zero.iter());
    let mut ct_bits = BitIteratorLE::new(ct.into_repr());

    let mut k = E::Fr::one();
    let mut acc = Signal::zero();

    for i in 0..nsteps {
        let ct_l = ct_bits.next().unwrap();
        let ct_u = ct_bits.next().unwrap();

        let sig_l = sig_bits.next().unwrap();
        let sig_u = sig_bits.next().unwrap();

        let sig_lu = sig_l.multiply(cs.namespace(|| format!("sig_l*sig_u; i={}", i)), &sig_u)?;

        acc = acc + &(k * &match (ct_l, ct_u) {
            (false, false) => -sig_lu + sig_l + sig_u,
            (true, false) => -sig_lu + sig_l + sig_u + sig_u - &Signal::one(),
            (false, true) => sig_lu + sig_u - &Signal::one(),
            (true, true) => sig_lu - &Signal::one()
        });
        k.double();
    }

    k.sub_assign(&E::Fr::one());

    acc = acc + &Signal::Constant(k);
    let acc_bits = into_bits_le(cs.namespace(|| "bitify acc"), &acc, nsteps+1)?;
    Ok(acc_bits[nsteps].clone())
}


pub fn into_bits_le_strict<E:Engine, CS:ConstraintSystem<E>>(
    mut cs: CS,
    signal:&Signal<E>
) -> Result<Vec<Signal<E>>, SynthesisError>
    where CS: ConstraintSystem<E>
{
    let bits = into_bits_le(cs.namespace(|| "split to bits"), signal, E::Fr::NUM_BITS as usize)?;
    let mut minus_one = E::Fr::one();
    minus_one.negate();
    let cmp_res = comp_constant(cs.namespace(|| "cmp with minus one"), &bits, &minus_one)?;
    cmp_res.assert_zero(cs.namespace(||"should be <= -1"))?;
    Ok(bits)
}

