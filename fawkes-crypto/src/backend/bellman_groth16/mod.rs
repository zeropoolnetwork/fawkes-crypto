use crate::{
    circuit::cs::{CS, RCS},
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
};

#[cfg(feature = "borsh_support")]
use borsh::{BorshSerialize, BorshDeserialize};
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

use bellman::pairing::{CurveAffine, RawEncodable};
use std::io::Cursor;

pub mod engines;
#[cfg(feature = "rand_support")]
pub mod osrng;
pub mod prover;
#[cfg(feature = "rand_support")]
pub mod setup;
pub mod verifier;

pub trait Engine {
    type BE: bellman::pairing::Engine;
    type Fq: PrimeField;
    type Fr: PrimeField;
}

#[repr(transparent)]
struct BellmanCS<E: Engine>(RCS<E::Fr>);

pub fn num_to_bellman_fp<Fx: PrimeField, Fy: bellman::pairing::ff::PrimeField>(
    from: Num<Fx>,
) -> Fy {
    let buff = from.as_mont_uint().as_inner().as_ref();

    let mut to = Fy::char();
    let to_ref = to.as_mut();

    assert!(buff.len() == to_ref.len());

    to_ref
        .iter_mut()
        .zip(buff.iter())
        .for_each(|(a, b)| *a = *b);
    Fy::from_raw_repr(to).unwrap()
}

pub fn bellman_fp_to_num<Fx: PrimeField, Fy: bellman::pairing::ff::PrimeField>(
    from: Fy,
) -> Num<Fx> {
    let from_raw = from.into_raw_repr();
    let buff = from_raw.as_ref();

    let mut to = Num::ZERO;
    let to_inner: &mut <Fx::Inner as ff_uint::Uint>::Inner = to.as_mont_uint_mut().as_inner_mut();
    let to_ref = to_inner.as_mut();
    assert!(buff.len() == to_ref.len());
    to_ref
        .iter_mut()
        .zip(buff.iter())
        .for_each(|(a, b)| *a = *b);
    to
}

pub struct Parameters<E: Engine>(bellman::groth16::Parameters<E::BE>);

impl<E: Engine> Parameters<E> {
    pub fn get_vk(&self) -> verifier::VK<E> {
        verifier::VK::from_bellman(&self.0.vk)
    }
}

#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
pub struct G1Point<E: Engine>(Num<E::Fq>, Num<E::Fq>);

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
pub struct G2Point<E: Engine>((Num<E::Fq>, Num<E::Fq>), (Num<E::Fq>, Num<E::Fq>));

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

#[cfg(feature = "heavy_tests")]
#[cfg(test)]
mod bellman_groth16_test {
    use super::engines::Bn256;
    use super::setup::setup;
    use super::*;
    use crate::circuit::num::CNum;
    use crate::circuit::poseidon::{c_poseidon_merkle_proof_root, CMerkleProof};
    use crate::core::signal::Signal;
    use crate::core::sizedvec::SizedVec;
    use crate::engines::bn256::Fr;
    use crate::native::poseidon::{poseidon_merkle_proof_root, MerkleProof, PoseidonParams};
    use crate::rand::{thread_rng, Rng};
    use ff_uint::PrimeField;

    #[test]
    fn test_circuit_poseidon_merkle_root() {
        fn circuit<Fr: PrimeField>(public: CNum<Fr>, secret: (CNum<Fr>, CMerkleProof<Fr, U32>)) {
            let poseidon_params = PoseidonParams::<Fr>::new(3, 8, 53);
            let res = c_poseidon_merkle_proof_root(&secret.0, &secret.1, &poseidon_params);
            res.assert_eq(&public);
        }
        let params = setup::<Bn256, _, _, _>(circuit);

        const PROOF_LENGTH: usize = 32;
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(3, 8, 53);
        let leaf = rng.gen();
        let sibling = (0..PROOF_LENGTH)
            .map(|_| rng.gen())
            .collect::<SizedVec<_, U32>>();
        let path = (0..PROOF_LENGTH)
            .map(|_| rng.gen())
            .collect::<SizedVec<bool, U32>>();
        let proof = MerkleProof { sibling, path };
        let root = poseidon_merkle_proof_root(leaf, &proof, &poseidon_params);

        let (inputs, snark_proof) = prover::prove(&params, &root, &(leaf, proof), circuit);

        let res = verifier::verify(&params.get_vk(), &snark_proof, &inputs);
        assert!(res, "Verifier result should be true");
    }
}
