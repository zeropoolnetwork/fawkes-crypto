use std::{
    io,
    io::{Read, Write},
};

use bellman::{groth16::VerifyingKey, SynthesisError};
use pairing::{bls12_381, bn256, CurveAffine, CurveProjective, EncodedPoint, Engine};
use serde::{Deserialize, Serialize};

use crate::{
    core::field::{AbstractField, Field, PrimeField},
    helpers::groth16::{prover::Proof, G1PointData, G2PointData},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct TruncatedVerifyingKeyData<F: Field> {
    pub alpha_g1: G1PointData<F>,
    pub beta_g2: G2PointData<F>,
    pub gamma_g2: G2PointData<F>,
    pub delta_g2: G2PointData<F>,
    pub ic: Vec<G1PointData<F>>,
}

#[derive(Clone)]
pub struct TruncatedVerifyingKey<E: Engine> {
    pub alpha_g1: E::G1Affine,
    pub beta_g2: E::G2Affine,
    pub gamma_g2: E::G2Affine,
    pub delta_g2: E::G2Affine,
    pub ic: Vec<E::G1Affine>,
}

impl TruncatedVerifyingKey<bn256::Bn256> {
    pub fn into_data(&self) -> TruncatedVerifyingKeyData<bn256::Fq> {
        TruncatedVerifyingKeyData {
            alpha_g1: G1PointData::from(self.alpha_g1),
            beta_g2: G2PointData::from(self.beta_g2),
            gamma_g2: G2PointData::from(self.gamma_g2),
            delta_g2: G2PointData::from(self.delta_g2),
            ic: self.ic.iter().map(|e| G1PointData::from(*e)).collect(),
        }
    }

    pub fn from_data(vk: &TruncatedVerifyingKeyData<bn256::Fq>) -> Self {
        Self {
            alpha_g1: Into::<bn256::G1Affine>::into(vk.alpha_g1),
            beta_g2: Into::<bn256::G2Affine>::into(vk.beta_g2),
            gamma_g2: Into::<bn256::G2Affine>::into(vk.gamma_g2),
            delta_g2: Into::<bn256::G2Affine>::into(vk.delta_g2),
            ic: vk
                .ic
                .iter()
                .map(|p| Into::<bn256::G1Affine>::into(*p))
                .collect(),
        }
    }
}

impl TruncatedVerifyingKey<bls12_381::Bls12> {
    pub fn into_data(&self) -> TruncatedVerifyingKeyData<bls12_381::Fq> {
        TruncatedVerifyingKeyData {
            alpha_g1: G1PointData::from(self.alpha_g1),
            beta_g2: G2PointData::from(self.beta_g2),
            gamma_g2: G2PointData::from(self.gamma_g2),
            delta_g2: G2PointData::from(self.delta_g2),
            ic: self.ic.iter().map(|e| G1PointData::from(*e)).collect(),
        }
    }

    pub fn from_data(vk: &TruncatedVerifyingKeyData<bls12_381::Fq>) -> Self {
        Self {
            alpha_g1: Into::<bls12_381::G1Affine>::into(vk.alpha_g1),
            beta_g2: Into::<bls12_381::G2Affine>::into(vk.beta_g2),
            gamma_g2: Into::<bls12_381::G2Affine>::into(vk.gamma_g2),
            delta_g2: Into::<bls12_381::G2Affine>::into(vk.delta_g2),
            ic: vk
                .ic
                .iter()
                .map(|p| Into::<bls12_381::G1Affine>::into(*p))
                .collect(),
        }
    }
}

impl<E: Engine> TruncatedVerifyingKey<E> {
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        writer.write_all(self.alpha_g1.into_compressed().as_ref())?;
        writer.write_all(self.beta_g2.into_compressed().as_ref())?;
        writer.write_all(self.gamma_g2.into_compressed().as_ref())?;
        writer.write_all(self.delta_g2.into_compressed().as_ref())?;
        for ic in &self.ic {
            writer.write_all(ic.into_compressed().as_ref())?;
        }
        Ok(())
    }

    pub fn read<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut g1_repr = <E::G1Affine as CurveAffine>::Compressed::empty();
        let mut g2_repr = <E::G2Affine as CurveAffine>::Compressed::empty();

        reader.read_exact(g1_repr.as_mut())?;
        let alpha_g1 = g1_repr
            .into_affine()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .and_then(|e| {
                if e.is_zero() {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "point at infinity",
                    ))
                } else {
                    Ok(e)
                }
            })?;

        reader.read_exact(g2_repr.as_mut())?;
        let beta_g2 = g2_repr
            .into_affine()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .and_then(|e| {
                if e.is_zero() {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "point at infinity",
                    ))
                } else {
                    Ok(e)
                }
            })?;

        reader.read_exact(g2_repr.as_mut())?;
        let gamma_g2 = g2_repr
            .into_affine()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .and_then(|e| {
                if e.is_zero() {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "point at infinity",
                    ))
                } else {
                    Ok(e)
                }
            })?;

        reader.read_exact(g2_repr.as_mut())?;
        let delta_g2 = g2_repr
            .into_affine()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .and_then(|e| {
                if e.is_zero() {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "point at infinity",
                    ))
                } else {
                    Ok(e)
                }
            })?;

        let mut ic = vec![];

        while reader.read_exact(g1_repr.as_mut()).is_ok() {
            let g1 = g1_repr
                .into_affine()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                .and_then(|e| {
                    if e.is_zero() {
                        Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "point at infinity",
                        ))
                    } else {
                        Ok(e)
                    }
                })?;
            ic.push(g1);
        }

        Ok(TruncatedVerifyingKey {
            alpha_g1: alpha_g1,
            beta_g2: beta_g2,
            gamma_g2: gamma_g2,
            delta_g2: delta_g2,
            ic: ic.clone(),
        })
    }
}

pub fn truncate_verifying_key<E: Engine>(vk: &VerifyingKey<E>) -> TruncatedVerifyingKey<E> {
    TruncatedVerifyingKey {
        alpha_g1: vk.alpha_g1.clone(),
        beta_g2: vk.beta_g2.clone(),
        gamma_g2: vk.gamma_g2.clone(),
        delta_g2: vk.delta_g2.clone(),
        ic: vk.ic.clone(),
    }
}

pub fn verify<'a, E: Engine>(
    tvk: &'a TruncatedVerifyingKey<E>,
    proof: &Proof<E>,
    public_inputs: &[E::Fr],
) -> Result<bool, SynthesisError> {
    if (public_inputs.len() + 1) != tvk.ic.len() {
        return Err(SynthesisError::MalformedVerifyingKey);
    }

    let mut acc = tvk.ic[0].into_projective();

    for (i, b) in public_inputs.iter().zip(tvk.ic.iter().skip(1)) {
        acc.add_assign(&b.mul(i.into_repr()));
    }

    // The original verification equation is:
    // A * B = alpha * beta + inputs * gamma + C * delta
    // ... however, we rearrange it so that it is:
    // (-A) * B + alpha * beta + inputs * gamma + C * delta == 1

    let mut neg_a = proof.0.a.clone();
    neg_a.negate();

    Ok(E::final_exponentiation(&E::miller_loop(
        [
            (&neg_a.prepare(), &proof.0.b.prepare()),
            (&tvk.alpha_g1.prepare(), &tvk.beta_g2.prepare()),
            (&acc.into_affine().prepare(), &tvk.gamma_g2.prepare()),
            (&proof.0.c.prepare(), &tvk.delta_g2.prepare()),
        ]
        .iter(),
    ))
    .unwrap()
        == E::Fqk::one())
}
