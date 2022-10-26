#[cfg(feature = "borsh_support")]
use borsh::{BorshSerialize, BorshDeserialize};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

use bellman::pairing::{CurveAffine, RawEncodable};
use std::io::Cursor;
use super::engines::Engine;
use ff_uint::Num;

#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct G1Point<E: Engine>(pub Num<E::Fq>, pub Num<E::Fq>);

#[cfg(feature = "borsh_support")]
impl<E: Engine> BorshSerialize for G1Point<E> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.0, writer)?;
        BorshSerialize::serialize(&self.1, writer)
    }
}

#[cfg(feature = "borsh_support")]
impl<E: Engine> BorshDeserialize for G1Point<E> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let x = BorshDeserialize::deserialize(buf)?;
        let y = BorshDeserialize::deserialize(buf)?;
        Ok(Self(x, y))
    }
}

#[cfg(feature = "borsh_support")]
impl<E: Engine> BorshSerialize for G2Point<E> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.0.0, writer)?;
        BorshSerialize::serialize(&self.0.1, writer)?;
        BorshSerialize::serialize(&self.1.0, writer)?;
        BorshSerialize::serialize(&self.1.1, writer)
    }
}

#[cfg(feature = "borsh_support")]
impl<E: Engine> BorshDeserialize for G2Point<E> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let x_re = BorshDeserialize::deserialize(buf)?;
        let x_im = BorshDeserialize::deserialize(buf)?;
        let y_re = BorshDeserialize::deserialize(buf)?;
        let y_im = BorshDeserialize::deserialize(buf)?;
        Ok(Self((x_re, x_im), (y_re, y_im)))
    }
}

impl<E: Engine> G1Point<E> {
    pub fn to_bellman(&self) -> <E::BE as bellman::pairing::Engine>::G1Affine {
        if self.0 == Num::ZERO && self.1 == Num::ZERO {
            <E::BE as bellman::pairing::Engine>::G1Affine::zero()
        } else {
            let mut buf =
                <E::BE as bellman::pairing::Engine>::G1Affine::zero().into_raw_uncompressed_le();
            {
                let mut cur = Cursor::new(buf.as_mut());
                BorshSerialize::serialize(&self.0.to_mont_uint(), &mut cur).unwrap();
                BorshSerialize::serialize(&self.1.to_mont_uint(), &mut cur).unwrap();
            }
            <E::BE as bellman::pairing::Engine>::G1Affine::from_raw_uncompressed_le(&buf, false)
                .unwrap()
        }
    }

    pub fn from_bellman(g1: &<E::BE as bellman::pairing::Engine>::G1Affine) -> Self {
        if g1.is_zero() {
            Self(Num::ZERO, Num::ZERO)
        } else {
            let buf = g1.into_raw_uncompressed_le();
            let mut cur = buf.as_ref();
            let x = Num::from_mont_uint_unchecked(BorshDeserialize::deserialize(&mut cur).unwrap());
            let y = Num::from_mont_uint_unchecked(BorshDeserialize::deserialize(&mut cur).unwrap());
            Self(x, y)
        }
    }
}

// Complex components are listed in LE notation, X+IY
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct G2Point<E: Engine>(pub (Num<E::Fq>, Num<E::Fq>), pub (Num<E::Fq>, Num<E::Fq>));

impl<E: Engine> G2Point<E> {
    pub fn to_bellman(&self) -> <E::BE as bellman::pairing::Engine>::G2Affine {
        if self.0 .0 == Num::ZERO
            && self.0 .1 == Num::ZERO
            && self.1 .0 == Num::ZERO
            && self.1 .1 == Num::ZERO
        {
            <E::BE as bellman::pairing::Engine>::G2Affine::zero()
        } else {
            let mut buf =
                <E::BE as bellman::pairing::Engine>::G2Affine::zero().into_raw_uncompressed_le();
            {
                let mut cur = Cursor::new(buf.as_mut());
                BorshSerialize::serialize(&self.0.0.to_mont_uint(), &mut cur).unwrap();
                BorshSerialize::serialize(&self.0.1.to_mont_uint(), &mut cur).unwrap();
                BorshSerialize::serialize(&self.1.0.to_mont_uint(), &mut cur).unwrap();
                BorshSerialize::serialize(&self.1.1.to_mont_uint(), &mut cur).unwrap();
            }
            <E::BE as bellman::pairing::Engine>::G2Affine::from_raw_uncompressed_le(&buf, false)
                .unwrap()
        }
    }

    pub fn from_bellman(g2: &<E::BE as bellman::pairing::Engine>::G2Affine) -> Self {
        if g2.is_zero() {
            Self((Num::ZERO, Num::ZERO), (Num::ZERO, Num::ZERO))
        } else {
            let buf = g2.into_raw_uncompressed_le();
            let mut cur = buf.as_ref();
            let x_re = Num::from_mont_uint_unchecked(BorshDeserialize::deserialize(&mut cur).unwrap());
            let x_im = Num::from_mont_uint_unchecked(BorshDeserialize::deserialize(&mut cur).unwrap());
            let y_re = Num::from_mont_uint_unchecked(BorshDeserialize::deserialize(&mut cur).unwrap());
            let y_im = Num::from_mont_uint_unchecked(BorshDeserialize::deserialize(&mut cur).unwrap());
            Self((x_re, x_im), (y_re, y_im))
        }
    }
}
