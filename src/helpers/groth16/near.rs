use borsh::{BorshSerialize, BorshDeserialize};
use std::io::{self, Write};
use crate::core::field::Field;
use crate::native::num::Num;

use crate::helpers::groth16::{prover::ProofData, verifier::TruncatedVerifyingKeyData, G1PointData, G2PointData};


impl<T:Field> BorshSerialize for G1PointData<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.0.serialize(writer)?;
        self.1.serialize(writer)
    }
}

impl<T:Field> BorshDeserialize for G1PointData<T> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, io::Error> {
        Ok(Self(Num::deserialize(buf)?, Num::deserialize(buf)?))
    }
}

impl<T:Field> BorshSerialize for G2PointData<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.0 .1.serialize(writer)?;
        self.0 .0.serialize(writer)?;
        self.1 .1.serialize(writer)?;
        self.1 .0.serialize(writer)
    }
}

impl<T:Field> BorshDeserialize for G2PointData<T> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, io::Error> {
        let x_re = Num::deserialize(buf)?;
        let x_im = Num::deserialize(buf)?;
        let y_re = Num::deserialize(buf)?;
        let y_im = Num::deserialize(buf)?;
        Ok(Self((x_im, x_re), (y_im, y_re)))
    }
}

impl<T:Field> BorshSerialize for ProofData<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.a.serialize(writer)?;
        self.b.serialize(writer)?;
        self.c.serialize(writer)
    }
}

impl<T:Field> BorshDeserialize for ProofData<T> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, io::Error> {
        Ok(Self{
            a: G1PointData::deserialize(buf)?, 
            b: G2PointData::deserialize(buf)?,
            c: G1PointData::deserialize(buf)?
        })
    }
}


impl<T:Field> BorshSerialize for TruncatedVerifyingKeyData<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        self.alpha_g1.serialize(writer)?;
        self.beta_g2.serialize(writer)?;
        self.gamma_g2.serialize(writer)?;
        self.delta_g2.serialize(writer)?;
        self.ic.serialize(writer)
    }
}

impl<T:Field> BorshDeserialize for TruncatedVerifyingKeyData<T> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, io::Error> {
        Ok(Self{
            alpha_g1: G1PointData::deserialize(buf)?, 
            beta_g2: G2PointData::deserialize(buf)?,
            gamma_g2: G2PointData::deserialize(buf)?,
            delta_g2: G2PointData::deserialize(buf)?,
            ic : <_>::deserialize(buf)?
        })
    }
}
