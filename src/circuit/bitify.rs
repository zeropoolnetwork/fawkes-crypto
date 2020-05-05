
use crate::core::field::{PrimeField};

use crate::circuit::num::{CNum};
use crate::circuit::bool::{CBool};

use crate::core::signal::Signal;
use crate::native::num::Num;
use crate::core::cs::ConstraintSystem;



pub fn c_into_bits_le<'a, CS:ConstraintSystem>(
    signal:&CNum<'a, CS>,
    limit: usize
) -> Vec<CBool<'a, CS>>
{
    match signal.as_const() {
        Some(value) => {
            let mut bits = Vec::<CBool<'a, CS>>::new();
            let mut k = Num::one();
            let mut remained_value = value.clone();
            let value_bits = value.iterbit_le().collect::<Vec<_>>();
            for i in 0..limit {
                let bit = value_bits[i];
                if bit {
                    remained_value -= k;
                }
                bits.push(signal.derive_const(&bit) );
                k=k.double();
            }
            assert!(remained_value.is_zero());
            bits
        },
        _ => {
            let value = signal.get_value();
            let mut remained_signal = signal.clone();
            let mut k = Num::one();
            let mut bits = vec![signal.derive_const(&false); limit];
            let value_bits = match value {
                Some(v) => v.iterbit_le().map(|x| Some(x)).collect::<Vec<_>>(),
                None => vec![None; CS::F::NUM_BITS as usize]
            };

            for i in 1..limit {
                k=k.double();
                let s = signal.derive_alloc::<CBool<'a, CS>>(value_bits[i].as_ref());
                s.assert();
                remained_signal -= &s.0*k;
                bits[i] = s;    
            }

            remained_signal.assert_bit();
            bits[0]=remained_signal.into_bool();
            bits
        }
    }
}

// return 1 if signal > ct 
pub fn c_comp_constant<'a, CS:ConstraintSystem>(
    signal:&[CBool<'a, CS>],
    ct: Num<CS::F>
) -> CBool<'a, CS> {
    let siglen = signal.len();
    assert!(siglen>0, "should be at least one input signal");
    let cs = signal[0].get_cs();
    let nsteps = (siglen >> 1) + (siglen & 1);
    let sig_zero = if siglen & 1 == 1 {vec![CBool::c_false(cs)] } else {vec![]};

    let mut sig_bits = signal.iter().chain(sig_zero.iter());
    let mut ct_bits = ct.iterbit_le();

    let mut k = Num::one();
    let mut acc = CNum::zero(cs);

    for _ in 0..nsteps {
        let ct_l = ct_bits.next().unwrap();
        let ct_u = ct_bits.next().unwrap();

        let sig_l = sig_bits.next().unwrap();
        let sig_u = sig_bits.next().unwrap();

        let sig_lu = &sig_l.0*&sig_u.0;

        acc = acc + k * match (ct_l, ct_u) {
            (false, false) =>  &sig_l.0 + &sig_u.0 -sig_lu,
            (true, false) =>  &sig_l.0 + &sig_u.0 * num!(2) - sig_lu - Num::one(),
            (false, true) => sig_lu + &sig_u.0 - Num::one(),
            (true, true) => sig_lu - Num::one()
        };
        k=k.double();
    }

    k -= Num::one();

    acc = acc + k;
    let acc_bits = c_into_bits_le(&acc, nsteps+1);
    acc_bits[nsteps].clone()
}


pub fn c_into_bits_le_strict<'a, CS:ConstraintSystem>(
    signal:&CNum<'a, CS>
) -> Vec<CBool<'a, CS>>{
    let bits = c_into_bits_le(signal, CS::F::NUM_BITS as usize);
    let cmp_res = c_comp_constant( &bits, -Num::one());
    cmp_res.assert_false();
    bits
}

pub fn c_from_bits_le<'a, CS:ConstraintSystem>(
    bits:&[CNum<'a, CS>]
) -> CNum<'a, CS> {
    assert!(bits.len()>0, "should be positive number of bits");
    let mut acc = bits[0].clone();
    let mut k = Num::one();
    for i in 1..bits.len() {
        k = k.double();
        acc += k*&bits[i];
    }
    acc
}