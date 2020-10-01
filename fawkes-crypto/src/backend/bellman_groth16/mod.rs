

use ff_uint::{PrimeField, NumRepr, Num};
use crate::core::signal::Signal;
use crate::borsh::{BorshSerialize, BorshDeserialize};
use std::io::{Cursor};
use bellman::pairing::{CurveAffine, RawEncodable};

pub mod osrng;
pub mod prover;
pub mod verifier;

pub mod engines;

pub trait Engine {
    type BE: bellman::pairing::Engine;
    type Fq: PrimeField;
    type Fr: PrimeField;
}




pub fn num_to_bellman_fp<Fx:PrimeField, Fy:bellman::pairing::ff::PrimeField>(from:Num<Fx>) -> Fy {
    let buff = from.as_mont_uint().as_inner().as_ref();

    let mut to = Fy::char();
    let to_ref = to.as_mut();

    assert!(buff.len()==to_ref.len());
    
    to_ref.iter_mut().zip(buff.iter()).for_each(|(a,b)| *a=*b);
    Fy::from_raw_repr(to).unwrap()
}


pub fn bellman_fp_to_num<Fx:PrimeField, Fy:bellman::pairing::ff::PrimeField>(from:Fy) -> Num<Fx> {
    let from_raw = from.into_raw_repr();
    let buff = from_raw.as_ref();

    let mut to = Num::ZERO;
    let to_inner:&mut <Fx::Inner as ff_uint::Uint>::Inner = to.as_mont_uint_mut().as_inner_mut();
    let to_ref = to_inner.as_mut();
    assert!(buff.len()==to_ref.len());
    to_ref.iter_mut().zip(buff.iter()).for_each(|(a,b)| *a=*b);
    to
}



pub struct Parameters<E:Engine>(bellman::groth16::Parameters<E::BE>);
pub struct G1Point<E:Engine>(Num<E::Fq>, Num<E::Fq>);

impl<E:Engine> BorshSerialize for G1Point<E> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.serialize(writer)?;
        self.1.serialize(writer)
    }
}

impl<E:Engine> BorshDeserialize for G1Point<E> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let x = Num::deserialize(buf)?;
        let y = Num::deserialize(buf)?;
        Ok(Self(x,y))
    }
}

impl<E:Engine> BorshSerialize for G2Point<E> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.0.0.serialize(writer)?;
        self.0.1.serialize(writer)?;
        self.1.0.serialize(writer)?;
        self.1.1.serialize(writer)
    }
}

impl <E:Engine> BorshDeserialize for G2Point<E> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let x_re = Num::deserialize(buf)?;
        let x_im = Num::deserialize(buf)?;
        let y_re = Num::deserialize(buf)?;
        let y_im = Num::deserialize(buf)?;
        Ok(Self((x_re, x_im), (y_re, y_im)))
    }
}


impl<E:Engine> G1Point<E> {
    pub fn to_bellman(&self) -> <E::BE as bellman::pairing::Engine>::G1Affine {
        if self.0==Num::ZERO && self.1==Num::ZERO {
            <E::BE as bellman::pairing::Engine>::G1Affine::zero()
        } else {
            let mut buf = <E::BE as bellman::pairing::Engine>::G1Affine::zero().into_raw_uncompressed_le();
            {
                let mut cur = Cursor::new(buf.as_mut());
                self.0.to_mont_uint().serialize(&mut cur).unwrap();
                self.1.to_mont_uint().serialize(&mut cur).unwrap();
            }
            <E::BE as bellman::pairing::Engine>::G1Affine::from_raw_uncompressed_le(&buf, false).unwrap()
        }
    }

    pub fn from_bellman(g1:&<E::BE as bellman::pairing::Engine>::G1Affine) -> Self {
        if g1.is_zero() {
            Self(Num::ZERO, Num::ZERO)
        } else {
            let buf = g1.into_raw_uncompressed_le();
            let mut cur=buf.as_ref();
            let x = Num::from_mont_uint_unchecked(NumRepr::deserialize(&mut cur).unwrap());
            let y = Num::from_mont_uint_unchecked(NumRepr::deserialize(&mut cur).unwrap());
            Self(x,y)
        }
    }
}

// Complex components are listed in LE notation, X+IY
pub struct G2Point<E:Engine>(
    (Num<E::Fq>, Num<E::Fq>),
    (Num<E::Fq>, Num<E::Fq>)
);

impl<E:Engine> G2Point<E> {
    pub fn to_bellman(&self) -> <E::BE as bellman::pairing::Engine>::G2Affine {
        if self.0.0==Num::ZERO && self.0.1==Num::ZERO && self.1.0==Num::ZERO && self.1.1==Num::ZERO {
            <E::BE as bellman::pairing::Engine>::G2Affine::zero()
        } else {
            let mut buf = <E::BE as bellman::pairing::Engine>::G2Affine::zero().into_raw_uncompressed_le();
            {
                let mut cur = Cursor::new(buf.as_mut());
                self.0.0.to_mont_uint().serialize(&mut cur).unwrap();
                self.0.1.to_mont_uint().serialize(&mut cur).unwrap();
                self.1.0.to_mont_uint().serialize(&mut cur).unwrap();
                self.1.1.to_mont_uint().serialize(&mut cur).unwrap();
            }
            <E::BE as bellman::pairing::Engine>::G2Affine::from_raw_uncompressed_le(&buf, false).unwrap()
        }
    }

    pub fn from_bellman(g2:&<E::BE as bellman::pairing::Engine>::G2Affine) -> Self {
        if g2.is_zero() {
            Self((Num::ZERO, Num::ZERO),(Num::ZERO, Num::ZERO))
        } else {
            let buf = g2.into_raw_uncompressed_le();
            let mut cur=buf.as_ref();
            let x_re = Num::from_mont_uint_unchecked(NumRepr::deserialize(&mut cur).unwrap());
            let x_im = Num::from_mont_uint_unchecked(NumRepr::deserialize(&mut cur).unwrap());
            let y_re = Num::from_mont_uint_unchecked(NumRepr::deserialize(&mut cur).unwrap());
            let y_im = Num::from_mont_uint_unchecked(NumRepr::deserialize(&mut cur).unwrap());
            Self((x_re, x_im),(y_re, y_im))
        }
    }
}



