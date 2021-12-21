use crate::{
    circuit::{
        cs::{RCS, WitnessCS, CS},
        lc::{Index}
    },
    core::signal::Signal,
    ff_uint::{Num, PrimeField},
};

use bit_vec::BitVec;


use bellman::{ConstraintSystem, SynthesisError};

#[cfg(feature = "borsh_support")]
use borsh::{BorshSerialize, BorshDeserialize};


use bellman::pairing::CurveAffine;
use std::marker::PhantomData;
use engines::Engine;

pub mod engines;
#[cfg(feature = "rand_support")]
pub mod osrng;
pub mod prover;
#[cfg(feature = "rand_support")]
pub mod setup;
pub mod verifier;
pub mod group;



#[repr(transparent)]
pub struct BellmanCS<E: Engine, C:CS<Fr=E::Fr>>(RCS<C>, PhantomData<E>);

impl<E: Engine, C:CS<Fr=E::Fr>> BellmanCS<E,C> {
    pub fn new(inner:RCS<C>) -> Self {
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
            Index::Input(i)=>variables_input[i as usize],
            Index::Aux(i)=>variables_aux[i as usize],
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

        let mut variables_input = Vec::with_capacity(num_input);
        let mut variables_aux = Vec::with_capacity(num_aux);

        variables_input.push(BCS::one());
        for i in 1..num_input as u32 {
            let v = bellman_cs.alloc_input(
                || format!("input_{}", i),
                || cs.get_value(Index::Input(i)).map(num_to_bellman_fp).ok_or(SynthesisError::AssignmentMissing)
            ).unwrap();
            variables_input.push(v);
        }

        for i in 0..num_aux as u32{
            let v = bellman_cs.alloc(
                || format!("aux_{}", i),
                || cs.get_value(Index::Aux(i)).map(num_to_bellman_fp).ok_or(SynthesisError::AssignmentMissing)
            ).unwrap();
            variables_aux.push(v);
        }

        
        for (i,g) in cs.get_gate_iterator().enumerate() {
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

pub struct Parameters<E: Engine>(pub bellman::groth16::Parameters<E::BE>, pub u32, pub Vec<u8>, pub BitVec);

impl<E: Engine> Parameters<E> {
    pub fn get_vk(&self) -> verifier::VK<E> {
        verifier::VK::from_bellman(&self.0.vk)
    }

    pub fn get_witness_rcs(&self)->RCS<WitnessCS<E::Fr>> {
        WitnessCS::rc_new(self.1 as usize, &self.2, &self.3)
    }

    pub fn write<W:std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let bitvec_len = self.3.len() as u32;
        BorshSerialize::serialize(&self.1, writer)?;
        BorshSerialize::serialize(&self.2, writer)?;
        BorshSerialize::serialize(&bitvec_len, writer)?;
        BorshSerialize::serialize(&self.3.to_bytes(), writer)?;
        self.0.write(writer)
    }

    pub fn read(reader: &mut &[u8], disallow_points_at_infinity: bool, checked: bool) -> std::io::Result<Self> {
        let e1 = BorshDeserialize::deserialize(reader)?;
        let e2 = BorshDeserialize::deserialize(reader)?;
        let e3_len = <u32 as BorshDeserialize>::deserialize(reader)? as usize;
        let e3_buf:Vec<u8> = BorshDeserialize::deserialize(reader)?;

        if e3_len > e3_buf.len() * 8 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "inconsistent bitvec length"));
        }

        let mut e3 = BitVec::from_bytes(&e3_buf);
        e3.truncate(e3_len);


        let e0 = bellman::groth16::Parameters::read(reader, disallow_points_at_infinity, checked)?;
        Ok(Self(e0, e1, e2, e3))
    }

}
