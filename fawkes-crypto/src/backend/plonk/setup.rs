use super::*;
use halo2_proofs::plonk::{keygen_pk, keygen_vk};

use std::{
    rc::Rc,
    cell::{RefCell}
};

use crate::circuit::cs::BuildCS;

#[derive(Clone, Debug)]
pub struct ProvingKey<E: Engine>(pub halo2_proofs::plonk::ProvingKey<<E::BE as halo2_curves::pairing::Engine>::G1Affine>);

#[derive(Clone, Debug)]
pub struct VK<E: Engine>(pub halo2_proofs::plonk::VerifyingKey<<E::BE as halo2_curves::pairing::Engine>::G1Affine>);

pub fn setup<'a, Pub: Signal<BuildCS<crate::engines::bn256::Fr>>, Sec: Signal<BuildCS<crate::engines::bn256::Fr>>, C: Fn(Pub, Sec)>(
    params: &'a Parameters<super::engines::Bn256>,
    circuit: C) -> (VK<super::engines::Bn256>, ProvingKey<super::engines::Bn256>) {
    let cs = BuildCS::<crate::engines::bn256::Fr>::new(false);
    let ref rcs = Rc::new(RefCell::new(cs));

    let signal_pub = Pub::alloc(rcs, None);
    signal_pub.inputize();
    let signal_sec = Sec::alloc(rcs, None);

    circuit(signal_pub, signal_sec);
    let bcs = HaloCS::<BuildCS<crate::engines::bn256::Fr>>::new(rcs.clone());

    let vk = keygen_vk(&params.0, &bcs).unwrap();
    let pk = keygen_pk(&params.0, vk.clone(), &bcs).unwrap();

    (VK(vk), ProvingKey(pk))
}
