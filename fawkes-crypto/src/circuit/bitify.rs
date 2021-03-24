use ff_uint::NumRepr;

use crate::{
    circuit::{bool::CBool, num::CNum},
    core::signal::Signal,
    ff_uint::{BitIterLE, Num, PrimeField},
};

pub fn c_into_bits_le<Fr: PrimeField>(signal: &CNum<Fr>, limit: usize) -> Vec<CBool<Fr>> {
    match signal.as_const() {
        Some(value) => {
            let mut bits = Vec::<CBool<Fr>>::new();
            let mut k = Num::ONE;
            let mut remained_value = value.clone();
            let value_bits = value.bit_iter_le().collect::<Vec<_>>();
            for i in 0..limit {
                let bit = value_bits[i];
                if bit {
                    remained_value -= k;
                }
                bits.push(signal.derive_const(&bit));
                k = k.double();
            }
            assert!(remained_value.is_zero());
            bits
        }
        _ => {
            let value = signal.get_value();
            let mut remained_signal = signal.clone();
            let mut k = Num::ONE;
            let mut bits = vec![signal.derive_const(&false); limit];
            let value_bits = match value {
                Some(v) => v.bit_iter_le().map(|x| Some(x)).collect::<Vec<_>>(),
                None => vec![None; Fr::MODULUS_BITS as usize],
            };

            for i in 1..limit {
                k = k.double();
                let s = signal.derive_alloc::<CBool<Fr>>(value_bits[i].as_ref());
                remained_signal -= s.to_num() * k;
                bits[i] = s;
            }

            bits[0] = remained_signal.to_bool();
            bits
        }
    }
}

// return true if s1 > s2
// assuming log2(s1) <= limit, log2(s2) <= limit
// TODO: optimize for constant cases
pub fn c_comp<Fr: PrimeField>(s1:&CNum<Fr>, s2:&CNum<Fr>, limit:usize) -> CBool<Fr> {
    let t = (NumRepr::ONE << (limit as u32)) - NumRepr::ONE;
    let t = Num::from_mont_uint_unchecked(t);
    let n = t + s1 - s2;
    c_into_bits_le(&n, limit+1)[limit].clone()
}

// return true if signal > ct
pub fn c_comp_constant<Fr: PrimeField>(signal: &[CBool<Fr>], ct: Num<Fr>) -> CBool<Fr> {
    let siglen = signal.len();
    assert!(siglen > 0, "should be at least one input signal");
    let cs = signal[0].get_cs();
    let nsteps = (siglen >> 1) + (siglen & 1);
    let sig_zero = if siglen & 1 == 1 {
        vec![CBool::from_const(cs, &false)]
    } else {
        vec![]
    };

    let mut sig_bits = signal.iter().chain(sig_zero.iter());
    let mut ct_bits = ct.bit_iter_le();

    let mut k = Num::ONE;
    let mut acc = CNum::from_const(cs, &Num::ZERO);

    for _ in 0..nsteps {
        let ct_l = ct_bits.next().unwrap();
        let ct_u = ct_bits.next().unwrap();

        let sig_l = sig_bits.next().unwrap().to_num();
        let sig_u = sig_bits.next().unwrap().to_num();

        let sig_lu = &sig_l * &sig_u;

        acc = acc
            + k * match (ct_l, ct_u) {
                (false, false) => &sig_l + &sig_u - sig_lu,
                (true, false) => &sig_l + &sig_u * Num::from(2) - sig_lu - Num::ONE,
                (false, true) => sig_lu + &sig_u - Num::ONE,
                (true, true) => sig_lu - Num::ONE,
            };
        k = k.double();
    }

    k -= Num::ONE;

    acc = acc + k;
    let acc_bits = c_into_bits_le(&acc, nsteps + 1);
    acc_bits[nsteps].clone()
}

pub fn c_into_bits_le_strict<Fr: PrimeField>(signal: &CNum<Fr>) -> Vec<CBool<Fr>> {
    let bits = c_into_bits_le(signal, Fr::MODULUS_BITS as usize);
    let cmp_res = c_comp_constant(&bits, -Num::ONE);
    cmp_res.assert_const(&false);
    bits
}

pub fn c_from_bits_le<Fr: PrimeField>(bits: &[CBool<Fr>]) -> CNum<Fr> {
    assert!(bits.len() > 0, "should be positive number of bits");
    let mut acc = bits[0].to_num();
    let mut k = Num::ONE;
    for i in 1..bits.len() {
        k = k.double();
        acc += k * bits[i].to_num();
    }
    acc
}
