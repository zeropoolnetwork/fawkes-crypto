use crate::{
    core::sizedvec::SizedVec,
    ff_uint::seedbox::SeedboxBlake2,
    ff_uint::{Num, PrimeField},
    rand::Rng,
    serde::{Deserialize, Serialize},
    typenum::Unsigned,
};

#[derive(Debug, Clone)]
pub struct PoseidonParams<Fr: PrimeField> {
    pub c: Vec<Vec<Num<Fr>>>,
    pub m: Vec<Vec<Num<Fr>>>,
    pub t: usize,
    pub f: usize,
    pub p: usize,
}

impl<Fr: PrimeField> PoseidonParams<Fr> {
    pub fn new(t: usize, f: usize, p: usize) -> Self {
        let mut seedbox = SeedboxBlake2::new_with_salt(
            format!("fawkes_poseidon(t={},f={},p={})", t, f, p).as_bytes(),
        );

        let c = (0..f + p)
            .map(|_| (0..t).map(|_| seedbox.gen()).collect())
            .collect();
        let m = (0..t)
            .map(|_| (0..t).map(|_| seedbox.gen()).collect())
            .collect();
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

pub fn poseidon<Fr: PrimeField>(inputs: &[Num<Fr>], params: &PoseidonParams<Fr>) -> Num<Fr> {
    let mut state = vec![Num::ZERO; params.t];
    let n_inputs = inputs.len();
    assert!(
        n_inputs <= params.t,
        "number of inputs should be less or equal than t"
    );
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    (&mut state[0..n_inputs]).clone_from_slice(inputs);

    let half_f = params.f >> 1;

    for i in 0..params.f + params.p {
        ark(&mut state, &params.c[i]);
        if i < half_f || i >= half_f + params.p {
            for j in 0..params.t {
                state[j] = sigma(state[j]);
            }
        } else {
            state[0] = sigma(state[0]);
        }
        mix(&mut state, params);
    }
    state[0]
}

pub fn poseidon_with_salt<Fr: PrimeField>(
    inputs: &[Num<Fr>],
    seed: &[u8],
    params: &PoseidonParams<Fr>,
) -> Num<Fr> {
    let n_inputs = inputs.len();
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    assert!(
        n_inputs < params.t,
        "number of inputs should be less than t"
    );
    let mut inputs = inputs.to_vec();
    inputs.push(Num::from_seed(seed));
    poseidon(&inputs, params)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MerkleProof<Fr: PrimeField, L: Unsigned> {
    pub sibling: SizedVec<Num<Fr>, L>,
    pub path: SizedVec<bool, L>,
}

pub fn poseidon_merkle_proof_root<Fr: PrimeField, L: Unsigned>(
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
