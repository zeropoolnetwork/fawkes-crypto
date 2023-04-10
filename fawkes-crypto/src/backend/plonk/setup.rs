use super::{*, engines::Bn256};
use halo2_curves::pairing::Engine as PairingEngine;
use crate::backend::plonk::engines::Engine;
use halo2_proofs::plonk::{
    keygen_pk,
    keygen_vk,
    VerifyingKey as HaloVerifyingKey,
    ProvingKey as HaloProvingKey,
};

use std::{
    rc::Rc,
    cell::{RefCell}
};

use crate::{circuit::cs::BuildCS, engines::bn256::Fr};

#[derive(Clone, Debug)]
pub struct ProvingKey<E: Engine>(
    pub HaloProvingKey<<E::BE as PairingEngine>::G1Affine>
);

#[derive(Clone, Debug)]
pub struct VerifyingKey<E: Engine>(
    pub HaloVerifyingKey<<E::BE as PairingEngine>::G1Affine>
);

pub fn setup<
    'a,
    Pub: Signal<BuildCS<Fr>>,
    Sec: Signal<BuildCS<Fr>>,
    C: Fn(Pub, Sec)
>(
    params: &'a Parameters<Bn256>,
    circuit: C
) -> (VerifyingKey<Bn256>, ProvingKey<Bn256>) {
    let cs = BuildCS::<Fr>::new(false);
    let ref rcs = Rc::new(RefCell::new(cs));

    let signal_pub = Pub::alloc(rcs, None);
    signal_pub.inputize();
    let signal_sec = Sec::alloc(rcs, None);

    circuit(signal_pub, signal_sec);
    let bcs = HaloCS::<BuildCS<Fr>>::new(rcs.clone());

    let vk = keygen_vk(&params.0, &bcs).unwrap();
    let pk = keygen_pk(&params.0, vk.clone(), &bcs).unwrap();

    (VerifyingKey(vk), ProvingKey(pk))
}
