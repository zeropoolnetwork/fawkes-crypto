use crate::{
    core::sizedvec::SizedVec,
    ff_uint::seedbox::{SeedboxBlake2, SeedBox, SeedBoxGen},
    ff_uint::{Num, PrimeField},
};

#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Serialize};

use itertools::Itertools;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "", deserialize = "")))]
pub struct PoseidonParams<Fr: PrimeField> {
    pub c: Vec<Vec<Num<Fr>>>,
    pub m: Vec<Vec<Num<Fr>>>,
    pub t: usize,
    pub f: usize,
    pub p: usize,
}

impl<Fr: PrimeField> PoseidonParams<Fr> {
    pub fn new(t: usize, f: usize, p: usize) -> Self {
        Self::new_with_salt(t, f, p, "")
    }

    pub fn new_with_salt(t: usize, f: usize, p: usize, salt:&str) -> Self {

        fn m<Fr: PrimeField>(n: usize, seedbox: &mut SeedboxBlake2) -> Vec<Vec<Num<Fr>>> {
            let x = (0..n).map(|_| seedbox.gen()).collect::<Vec<_>>();
            let y = (0..n).map(|_| seedbox.gen()).collect::<Vec<_>>();
            (0..n).map(|i| (0..n).map(|j| Num::ONE/(x[i] + y[j]) ).collect()).collect()
        }

        let mut seedbox = SeedboxBlake2::new_with_salt(
            format!("fawkes_poseidon(t={},f={},p={},salt={})", t, f, p, salt).as_bytes(),
        );

        let c = (0..f + p)
            .map(|_| (0..t).map(|_| seedbox.gen()).collect())
            .collect();
        let m = m(t, &mut seedbox);
        PoseidonParams { c, m, t, f, p }
    }
}

fn ark<Fr: PrimeField>(state: &mut [Num<Fr>], c: &[Num<Fr>]) {
    state.iter_mut().zip(c.iter()).for_each(|(s, c)| *s += c)
}

fn sigma<Fr: PrimeField>(a: Num<Fr>) -> Num<Fr> {
    a.square().square() * a
}

fn mix<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    let statelen = state.len();
    let mut new_state = vec![Num::ZERO; statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.m[i][j] * state[j];
        }
    }
    state.clone_from_slice(&new_state);
}

fn perm<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    assert!(state.len() == params.t);
    let half_f = params.f >> 1;

    for i in 0..params.f + params.p {
        ark(state, &params.c[i]);
        if i < half_f || i >= half_f + params.p {
            for j in 0..params.t {
                state[j] = sigma(state[j]);
            }
        } else {
            state[0] = sigma(state[0]);
        }
        mix(state, params);
    }
}

pub fn poseidon<Fr: PrimeField>(inputs: &[Num<Fr>], params: &PoseidonParams<Fr>) -> Num<Fr> {
    let mut state = vec![Num::ZERO; params.t];
    let n_inputs = inputs.len();
    assert!(
        n_inputs < params.t,
        "number of inputs should be less or equal than t"
    );
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    (&mut state[0..n_inputs]).clone_from_slice(inputs);

    perm(&mut state, params);
    state[0]
}

pub fn poseidon_sponge<Fr: PrimeField>(inputs: &[Num<Fr>], params: &PoseidonParams<Fr>) -> Num<Fr> {
    let mut state = vec![Num::ZERO; params.t];
    let size = Num::from(inputs.len() as u64);
    core::iter::once(&size).chain(inputs.iter()).chunks(params.t-1).into_iter().for_each(|c| {
        state.iter_mut().zip(c.into_iter()).for_each(|(l, r)| *l+=*r);
        perm(&mut state, params);
    });
    state[0]
}


#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "", deserialize = "")))]
pub struct MerkleProof<Fr: PrimeField, const L: usize> {
    pub sibling: SizedVec<Num<Fr>, L>,
    pub path: SizedVec<bool, L>,
}

pub fn poseidon_merkle_proof_root<Fr: PrimeField, const L: usize>(
    leaf: Num<Fr>,
    proof: &MerkleProof<Fr, L>,
    params: &PoseidonParams<Fr>,
) -> Num<Fr> {
    let mut root = leaf.clone();
    for (&p, &s) in proof.path.iter().zip(proof.sibling.iter()) {
        let pair = if p { [s, root] } else { [root, s] };
        root = poseidon(pair.as_ref(), params);
    }
    root
}

pub fn poseidon_merkle_tree_root<Fr: PrimeField>(
    leaf: &[Num<Fr>],
    params: &PoseidonParams<Fr>,
) -> Num<Fr> {
    let leaf_sz = leaf.len();
    assert!(leaf_sz > 0, "should be at least one leaf in the tree");
    let proof_sz = std::mem::size_of::<usize>() * 8 - (leaf_sz - 1).leading_zeros() as usize;
    let total_leaf_sz = 1usize << proof_sz;
    let mut state = leaf.to_vec();
    state.extend_from_slice(&vec![Num::ZERO; total_leaf_sz - leaf_sz]);
    for j in 0..proof_sz {
        for i in 0..total_leaf_sz >> (j + 1) {
            state[i] = poseidon(&[state[2 * i].clone(), state[2 * i + 1].clone()], params);
        }
    }
    state[0].clone()
}
