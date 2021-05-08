use crate::{
    circuit::{
        cs::{RCS, WitnessCS, CS, Gate},
        lc::{Index}
    },
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
};

use bit_vec::BitVec;

use bellman::{ConstraintSystem, SynthesisError};

#[cfg(feature = "borsh_support")]
use borsh::{BorshSerialize, BorshDeserialize};
#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};

use bellman::pairing::{CurveAffine, RawEncodable};
use std::{io::Cursor, marker::PhantomData};

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
struct BellmanCS<E: Engine, C:CS<Fr=E::Fr>>(RCS<C>, PhantomData<E>);

impl<E: Engine, C:CS<Fr=E::Fr>> BellmanCS<E,C> {
    fn new(inner:RCS<C>) -> Self {
        Self(inner, PhantomData)
    }
}

pub fn convert_lc<E: Engine>(
    lc: &[(Num<E::Fr>, Index)],
    variables_input: &[bellman::Variable],
    variables_aux: &[bellman::Variable],    
) -> bellman::LinearCombination<E::BE> {
    let mut res = Vec::with_capacity(lc.len());

    for e in lc.iter() {
        let k = num_to_bellman_fp(e.0);
        let v = match e.1 {
            Index::Input(i)=>variables_input[i],
            Index::Aux(i)=>variables_aux[i]
        };
        res.push((v, k));
    }
    bellman::LinearCombination::new(res)
}

impl<E: Engine, C:CS<Fr=E::Fr>> bellman::Circuit<E::BE> for BellmanCS<E, C> {
    fn synthesize<BCS: ConstraintSystem<E::BE>>(
        self,
        bellman_cs: &mut BCS,
    ) -> Result<(), SynthesisError> {
        let rcs = self.0;
        let cs = rcs.borrow();
        let num_input = cs.num_input();
        let num_aux = cs.num_aux();
        let num_gates = cs.num_gates();

        let mut variables_input = Vec::with_capacity(num_input);
        let mut variables_aux = Vec::with_capacity(num_aux);

        variables_input.push(BCS::one());
        for i in 1..num_input {
            let v = bellman_cs.alloc_input(
                || format!("input_{}", i),
                || cs.get_value(Index::Input(i)).map(num_to_bellman_fp).ok_or(SynthesisError::AssignmentMissing)
            ).unwrap();
            variables_input.push(v);
        }

        for i in 0..num_aux {
            let v = bellman_cs.alloc(
                || format!("aux_{}", i),
                || cs.get_value(Index::Aux(i)).map(num_to_bellman_fp).ok_or(SynthesisError::AssignmentMissing)
            ).unwrap();
            variables_aux.push(v);
        }

        
        
        for i in 0..num_gates {
            let g = cs.get_gate(i);
            bellman_cs.enforce(
                || format!("constraint {}", i),
                |_| convert_lc::<E>(&g.0, &variables_input, &variables_aux),
                |_| convert_lc::<E>(&g.1, &variables_input, &variables_aux),
                |_| convert_lc::<E>(&g.2, &variables_input, &variables_aux),
            );
        }
        Ok(())
    }
}


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

pub struct Parameters<E: Engine>(bellman::groth16::Parameters<E::BE>, Vec<Gate<E::Fr>>, BitVec);

impl<E: Engine> Parameters<E> {
    pub fn get_vk(&self) -> verifier::VK<E> {
        verifier::VK::from_bellman(&self.0.vk)
    }

    pub fn get_witness_rcs(&self)->RCS<WitnessCS<E::Fr>> {
        WitnessCS::rc_new(&self.1, &self.2)
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

    #[test]
    fn test_circuit_poseidon_merkle_root() {
        fn circuit<C:CS>(public: CNum<C>, secret: (CNum<C>, CMerkleProof<C, 32>)) {
            let poseidon_params = PoseidonParams::<C::Fr>::new(3, 8, 53);
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
            .collect::<SizedVec<_, 32>>();
        let path = (0..PROOF_LENGTH)
            .map(|_| rng.gen())
            .collect::<SizedVec<bool, 32>>();
        let proof = MerkleProof { sibling, path };
        let root = poseidon_merkle_proof_root(leaf, &proof, &poseidon_params);

        println!("BitVec length {}", params.2.len());

        let (inputs, snark_proof) = prover::prove(&params, &root, &(leaf, proof), circuit);

        let res = verifier::verify(&params.get_vk(), &snark_proof, &inputs);
        assert!(res, "Verifier result should be true");
    }
}
