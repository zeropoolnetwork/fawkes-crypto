use bit_vec::BitVec;
use wtns_file::{WtnsFile, FieldElement};

use crate::ff_uint::{Uint, PrimeField};
use crate::core::signal::Signal;
use crate::circuit::cs::{WitnessCS, Gate};

pub fn get_witness<
    'a,
    Fr: PrimeField,
    Pub: Signal<WitnessCS<'a, Fr>>,
    Sec: Signal<WitnessCS<'a, Fr>>,
    C: Fn(Pub, Sec),
    const FS: usize,
>(
    gates: &'a Vec<Gate<Fr>>,
    consts: &'a BitVec,
    input_pub: &'a Pub::Value,
    input_sec: &'a Sec::Value,
    circuit: C,
) -> WtnsFile<FS> {
    let cs = WitnessCS::rc_new(gates, consts);

    let mut prime_bytes = [0; FS];
    Fr::MODULUS.put_little_endian(&mut prime_bytes);

    let signal_pub = Pub::alloc(&cs, Some(input_pub));
    signal_pub.inputize();
    let signal_sec = Sec::alloc(&cs, Some(input_sec));

    circuit(signal_pub, signal_sec);

    let cs = cs.borrow();

    let witness = cs.values_input
        .iter()
        .chain(cs.values_aux.iter())
        .map(|num| {
            let mut bytes = [0; FS];
            num.0.to_uint().put_little_endian(&mut bytes);
            FieldElement::from(bytes)
        })
        .collect();
    
    WtnsFile::from_vec(witness, FieldElement::from(prime_bytes))
}
