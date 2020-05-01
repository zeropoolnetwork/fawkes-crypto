use pairing::{
    Engine,
    CurveProjective,
    CurveAffine,
    EncodedPoint
};

use ff::{PrimeField, Field};

use bellman::groth16::{
    Proof,
    VerifyingKey
};

use bellman::SynthesisError;

use std::io::{Read, Write};
use std::io;


#[derive(Clone)]
pub struct TruncatedVerifyingKey<E: Engine> {
    pub alpha_g1: E::G1Affine,
    pub beta_g2: E::G2Affine,
    pub gamma_g2: E::G2Affine,
    pub delta_g2: E::G2Affine,
    pub ic: Vec<E::G1Affine>
}


impl<E: Engine> TruncatedVerifyingKey<E> {
    pub fn write<W: Write>(
        &self,
        mut writer: W
    ) -> io::Result<()>
    {
        writer.write_all(self.alpha_g1.into_compressed().as_ref())?;
        writer.write_all(self.beta_g2.into_compressed().as_ref())?;
        writer.write_all(self.gamma_g2.into_compressed().as_ref())?;
        writer.write_all(self.delta_g2.into_compressed().as_ref())?;
        for ic in &self.ic {
            writer.write_all(ic.into_compressed().as_ref())?;
        }
        Ok(())
    }

    pub fn read<R: Read>(
        mut reader: R
    ) -> io::Result<Self>
    {
        let mut g1_repr = <E::G1Affine as CurveAffine>::Compressed::empty();
        let mut g2_repr = <E::G2Affine as CurveAffine>::Compressed::empty();

        reader.read_exact(g1_repr.as_mut())?;
        let alpha_g1 = g1_repr
                .into_affine()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                .and_then(|e| if e.is_zero() {
                    Err(io::Error::new(io::ErrorKind::InvalidData, "point at infinity"))
                } else {
                    Ok(e)
                })?;

        reader.read_exact(g2_repr.as_mut())?;
        let beta_g2 = g2_repr
                .into_affine()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                .and_then(|e| if e.is_zero() {
                    Err(io::Error::new(io::ErrorKind::InvalidData, "point at infinity"))
                } else {
                    Ok(e)
                })?;

        reader.read_exact(g2_repr.as_mut())?;
        let gamma_g2 = g2_repr
                .into_affine()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                .and_then(|e| if e.is_zero() {
                    Err(io::Error::new(io::ErrorKind::InvalidData, "point at infinity"))
                } else {
                    Ok(e)
                })?;

        reader.read_exact(g2_repr.as_mut())?;
        let delta_g2 = g2_repr
                .into_affine()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                .and_then(|e| if e.is_zero() {
                    Err(io::Error::new(io::ErrorKind::InvalidData, "point at infinity"))
                } else {
                    Ok(e)
                })?;

        let mut ic = vec![];

        while reader.read_exact(g1_repr.as_mut()).is_ok() {
            let g1 = g1_repr
                    .into_affine()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
                    .and_then(|e| if e.is_zero() {
                        Err(io::Error::new(io::ErrorKind::InvalidData, "point at infinity"))
                    } else {
                        Ok(e)
                    })?;
            ic.push(g1);
        }

        Ok(TruncatedVerifyingKey {
            alpha_g1: alpha_g1,
            beta_g2: beta_g2,
            gamma_g2: gamma_g2,
            delta_g2: delta_g2,
            ic: ic.clone()
        })
    }
}

pub fn truncate_verifying_key<E: Engine>(
    vk: &VerifyingKey<E>
) -> TruncatedVerifyingKey<E>
{
    TruncatedVerifyingKey {
        alpha_g1: vk.alpha_g1.clone(),
        beta_g2: vk.beta_g2.clone(),
        gamma_g2: vk.gamma_g2.clone(),
        delta_g2: vk.delta_g2.clone(),
        ic: vk.ic.clone()
    }
}

pub fn verify_proof<'a, E: Engine>(
    tvk: &'a TruncatedVerifyingKey<E>,
    proof: &Proof<E>,
    public_inputs: &[E::Fr]
) -> Result<bool, SynthesisError>
{
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

    let mut neg_a = proof.a.clone();
    neg_a.negate();

    Ok(E::final_exponentiation(
        &E::miller_loop([
            (&neg_a.prepare(), &proof.b.prepare()),
            (&tvk.alpha_g1.prepare(), &tvk.beta_g2.prepare()),
            (&acc.into_affine().prepare(), &tvk.gamma_g2.prepare()),
            (&proof.c.prepare(), &tvk.delta_g2.prepare())
        ].into_iter())
    ).unwrap() == E::Fqk::one())
}