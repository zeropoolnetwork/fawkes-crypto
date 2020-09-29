use typenum::Unsigned;
use ff_uint::{Num, PrimeField};
use crate::circuit::{
    num::CNum,
    bool::CBool,
    cs::{RCS}
};
use crate::core::{
    signal::Signal,
    sizedvec::SizedVec,
};
use crate::native::poseidon::{PoseidonParams, MerkleProof};


#[derive(Clone, Signal)]
#[Value="MerkleProof<Fr, L>"]
pub struct CMerkleProof<Fr:PrimeField, L:Unsigned> {
    pub sibling: SizedVec<CNum<Fr>, L>,
    pub path: SizedVec<CBool<Fr>, L>
}

fn ark<Fr: PrimeField>(state: &mut[CNum<Fr>], c:&[Num<Fr>]) {
    state.iter_mut().zip(c.iter()).for_each(|(e,c)| *e += c);
}

fn sigma<Fr: PrimeField>(a:&CNum<Fr>) -> CNum<Fr> {
    let a_sq = a*a;
    let a_quad = &a_sq*&a_sq;
    a_quad*a
}

fn mix<Fr: PrimeField>(state: &mut[CNum<Fr>], params:&PoseidonParams<Fr>) {
    let statelen = state.len();
    let cs = state[0].get_cs();
    let mut new_state = vec![CNum::from_const(cs, &Num::ZERO); statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.m[i][j] * &state[j];
        }
    }
    state.clone_from_slice(&new_state);
}


pub fn c_poseidon<Fr: PrimeField>(inputs:&[CNum<Fr>], params:&PoseidonParams<Fr>) -> CNum<Fr> {
    let n_inputs = inputs.len();
    assert!(n_inputs <= params.t, "number of inputs should be less or equal than t");
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    let cs = inputs[0].get_cs();
    let mut state = vec![CNum::from_const(cs, &Num::ZERO); params.t];
    (&mut state[0..n_inputs]).clone_from_slice(inputs);

    let half_f = params.f>>1;

    for i in 0..params.f+params.p {
        ark(&mut state, &params.c[i]);
        if i < half_f || i >= half_f + params.p {
            for j in 0..params.t {
                state[j] = sigma(&state[j]);
            }
        } else {
            state[0] = sigma(&state[0]);
        }
        mix(&mut state, params);
    }
    state[0].clone()
}

pub fn c_poseidon_with_salt<Fr: PrimeField>(inputs:&[CNum<Fr>], seed: &[u8], params:&PoseidonParams<Fr>) -> CNum<Fr> {
    let n_inputs = inputs.len();
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    assert!(n_inputs < params.t, "number of inputs should be less than t");
    let cs = inputs[0].get_cs();
    let mut inputs = inputs.to_vec();
    inputs.push(CNum::from_const(cs, &Num::from_seed(seed)));
    c_poseidon(&inputs, params)
}


pub fn c_poseidon_merkle_proof_root<Fr: PrimeField, L:Unsigned>(
    leaf:&CNum<Fr>, 
    proof:&CMerkleProof<Fr, L>,
    params:&PoseidonParams<Fr>
) -> CNum<Fr> {
    let mut root = leaf.clone();
    for (p, s) in proof.path.iter().zip(proof.sibling.iter()) {
        let first = s.switch(p, &root); 
        let second = &root + s - &first;
        root = c_poseidon( [first, second].as_ref(), params);
    }
    root
}

pub fn c_poseidon_merkle_tree_root<Fr: PrimeField>(leaf: &[CNum<Fr>], params: &PoseidonParams<Fr>) -> CNum<Fr> {
    let leaf_sz = leaf.len();
    assert!(leaf_sz>0, "should be at least one leaf in the tree");
    let cs = leaf[0].get_cs();
    let proof_sz = std::mem::size_of::<usize>() * 8 - (leaf_sz-1).leading_zeros() as usize;
    let total_leaf_sz = 1usize << proof_sz;
    let mut state = leaf.to_vec();
    state.extend_from_slice(&vec![CNum::from_const(cs, &Num::ZERO); total_leaf_sz-leaf_sz]);
    for j in 0..proof_sz {
        for i in 0..total_leaf_sz>>(j + 1) {
            state[i] = c_poseidon(&[state[2*i].clone(), state[2*i+1].clone()], params);
        }
    }
    state[0].clone()
}
