use bit_vec::BitVec;
use wtns_file::{WtnsFile, FieldElement};

use crate::ff_uint::{Uint, PrimeField};
use crate::core::signal::Signal;
use crate::circuit::cs::{WitnessCS, Gate, CS};
use crate::circuit::lc::Index;

pub fn get_witness<
    'a,
    Fr: PrimeField,
    Pub: Signal<WitnessCS<'a, Fr>>,
    Sec: Signal<WitnessCS<'a, Fr>>,
    const FS: usize,
>(
    gates: &'a Vec<Gate<Fr>>,
    consts: &'a BitVec,
    input_pub: &'a Pub::Value,
    input_sec: &'a Sec::Value,
) -> WtnsFile<FS> {
    let cs = WitnessCS::rc_new(gates, consts);

    let mut prime_bytes = [0; FS];
    Fr::MODULUS.put_little_endian(&mut prime_bytes);

    let signal_pub = Pub::alloc(&cs, Some(input_pub));
    signal_pub.inputize();
    let _signal_sec = Sec::alloc(&cs, Some(input_sec));

    let cs = cs.borrow_mut();
    let mut witness = Vec::with_capacity(cs.num_aux());

    for i in (cs.num_input() + 1)..cs.num_aux() {
        let num = cs.get_value(Index::Aux(i)).unwrap();
        let mut bytes = [0; FS];
        num.0.to_uint().put_little_endian(&mut bytes);
        let fe = FieldElement::from(bytes);

        witness.push(fe);
    }
    
    WtnsFile::from_vec(witness, FieldElement::from(prime_bytes))
}
