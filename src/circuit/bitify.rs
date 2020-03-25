use bellman::pairing::{
    Engine
};

use ff::{
    PrimeField
};

use bellman::{
    SynthesisError,
    ConstraintSystem
};



use super::signal::Signal;
use crate::bititerator::BitIteratorLE;
use crate::wrappedmath::Wrap;

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
            let mut k = Wrap::one();
            let mut bits = vec![Signal::zero(); limit];
            let value_bits = match value {
                Some(v) => BitIteratorLE::new(v.into_repr()).map(|e| Some(e)).collect::<Vec<_>>(),
                None => vec![None; E::Fr::NUM_BITS as usize]
            };

            for i in 1..limit {
                k=k.double();
                let s = Signal::alloc(cs.namespace(|| format!(":=bit[{}]", i)), value_bits[i].map(|b| Wrap::from(b)))?;
                s.assert_bit(cs.namespace(|| format!("bit[{}]", i)))?;
                remained_signal = remained_signal - k * &s;
                bits[i] = s;    
            }

            remained_signal.assert_bit(cs.namespace(|| "bit[0]"))?;
            bits[0]=remained_signal;
            Ok(bits)
        },
        Signal::Constant(value) => {
            let mut bits = Vec::<Signal<E>>::new();
            let mut k = Wrap::one();
            let mut remained_value = value.clone();
            let value_bits = BitIteratorLE::new(value.into_repr()).collect::<Vec<_>>();
            for i in 0..limit {
                let bit = Wrap::from(value_bits[i])*k; 
                remained_value -= bit;
                bits.push(Signal::Constant(bit));
                k=k.double();
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
    ct: Wrap<E::Fr>
) -> Result<Signal<E>, SynthesisError> {
    let siglen = signal.len();
    let nsteps = (siglen >> 1) + (siglen & 1);
    let sig_zero = if siglen & 1 == 1 {vec![Signal::<E>::zero()] } else {vec![]};

    let mut sig_bits = signal.iter().chain(sig_zero.iter());
    let mut ct_bits = BitIteratorLE::new(ct.into_repr());

    let mut k = Wrap::one();
    let mut acc = Signal::zero();

    for i in 0..nsteps {
        let ct_l = ct_bits.next().unwrap();
        let ct_u = ct_bits.next().unwrap();

        let sig_l = sig_bits.next().unwrap();
        let sig_u = sig_bits.next().unwrap();

        let sig_lu = sig_l.multiply(cs.namespace(|| format!("lu[{}]", i)), &sig_u)?;

        acc = acc + k * match (ct_l, ct_u) {
            (false, false) => -sig_lu + sig_l + sig_u,
            (true, false) => -sig_lu + sig_l + sig_u + sig_u - &Signal::one(),
            (false, true) => sig_lu + sig_u - &Signal::one(),
            (true, true) => sig_lu - &Signal::one()
        };
        k=k.double();
    }

    k -= Wrap::one();

    acc = acc + &Signal::Constant(k);
    let acc_bits = into_bits_le(cs.namespace(|| "bitify(acc)"), &acc, nsteps+1)?;
    Ok(acc_bits[nsteps].clone())
}


pub fn into_bits_le_strict<E:Engine, CS:ConstraintSystem<E>>(
    mut cs: CS,
    signal:&Signal<E>
) -> Result<Vec<Signal<E>>, SynthesisError>
    where CS: ConstraintSystem<E>
{
    let bits = into_bits_le(cs.namespace(|| "bitify"), signal, E::Fr::NUM_BITS as usize)?;
    let cmp_res = comp_constant(cs.namespace(|| "cmp"), &bits, Wrap::minusone())?;
    cmp_res.assert_zero(cs.namespace(||"assert"))?;
    Ok(bits)
}

