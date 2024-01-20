use super::{*, engines::Bn256};
use halo2_curves::{
    CurveAffine,
    ff::FromUniformBytes,
    group::prime::PrimeCurveAffine,
    pairing::Engine as PairingEngine,
};
use halo2_proofs::SerdeFormat;
use crate::{
    circuit::cs::BuildCS,
    engines::bn256::Fr,
    backend::plonk::engines::Engine
};
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
use std::io::{Read, Write};

#[derive(Clone, Debug)]
pub struct ProvingKey<E: Engine>(
    pub HaloProvingKey<<E::BE as PairingEngine>::G1Affine>
);

impl<E: Engine> ProvingKey<E>
where
    <<E as Engine>::BE as PairingEngine>::G1Affine: SerdeObject,
    <<<E as Engine>::BE as PairingEngine>::G1Affine as PrimeCurveAffine>::Scalar: SerdeObject + FromUniformBytes<64>,
{
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut w = brotli::CompressorWriter::new(writer, 4096, 9, 22);
        self.0.write(&mut w, SerdeFormat::Processed)
    }

    pub fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        let mut r = brotli::Decompressor::new(reader, 4096);
        Ok(Self(HaloProvingKey::<<E::BE as PairingEngine>::G1Affine>::read::<_, HaloCS<BuildCS<E::Fr>>>(&mut r, SerdeFormat::Processed)?))
    }
}

#[derive(Clone, Debug)]
pub struct VerifyingKey<E: Engine>(
    pub HaloVerifyingKey<<E::BE as PairingEngine>::G1Affine>
);

impl<E: Engine> VerifyingKey<E>
where
    <<E as Engine>::BE as PairingEngine>::G1Affine: SerdeObject,
    <<<E as Engine>::BE as PairingEngine>::G1Affine as PrimeCurveAffine>::Scalar: SerdeObject + FromUniformBytes<64>,
{
    pub fn write<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.write(writer, SerdeFormat::Processed)
    }

    pub fn read<R>(reader: &mut R) -> std::io::Result<Self>
        where
            R: std::io::Read,
    {
        Ok(Self(HaloVerifyingKey::<<E::BE as PairingEngine>::G1Affine>::read::<_, HaloCS<BuildCS<E::Fr>>>(reader, SerdeFormat::Processed)?))
    }
}

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
