use ff::{
    PrimeField
};

use rand::Rng;

use crate::core::seedbox::SeedboxBlake2;
use crate::core::num::Num;

#[derive(Debug)]
pub struct PoseidonParams<F:PrimeField> {
    pub c: Vec<Num<F>>, 
    pub m: Vec<Vec<Num<F>>>, 
    pub t: usize, 
    pub f: usize, 
    pub p: usize
}

impl<F:PrimeField> PoseidonParams<F> {
    pub fn new(t:usize, f:usize, p:usize) -> Self {
        let mut seedbox = SeedboxBlake2::new_with_salt(format!("fawkes_poseidon(t={},f={},p={})", t, f, p).as_bytes()
        );


        let c = (0..f+p).map(|_| seedbox.gen()).collect();
        let m = (0..t).map(|_| (0..t).map(|_| seedbox.gen()).collect()).collect();
        PoseidonParams {c, m, t, f, p}
    }
}




fn ark<F:PrimeField>(state: &mut[Num<F>], c:Num<F>) {
    state.iter_mut().for_each(|e| *e += c)
}

fn sigma<F:PrimeField>(a: Num<F>) -> Num<F> {
    a.square().square()*a
}

fn mix<F:PrimeField>(state: &mut[Num<F>], params:&PoseidonParams<F>) {
    let statelen = state.len();
    let mut new_state = vec![Num::zero(); statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.m[i][j] * state[j];
        }
    }
    state.clone_from_slice(&new_state);
}


pub fn poseidon<F:PrimeField>(inputs:&[Num<F>], params:&PoseidonParams<F>) -> Num<F> {
    let mut state = vec![Num::zero(); params.t];
    let n_inputs = inputs.len();
    assert!(n_inputs <= params.t, "number of inputs should be less or equal than t");
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    (&mut state[0..n_inputs]).clone_from_slice(inputs);

    let half_f = params.f>>1;

    for i in 0..params.f+params.p {
        ark(&mut state, params.c[i]);
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


#[derive(Debug, Clone)]
pub struct MerkleProof<F:PrimeField> {
    pub sibling: Vec<Num<F>>,
    pub path: Vec<bool>
}


pub fn poseidon_merkle_proof_root<F:PrimeField>(leaf:Num<F>, proof:&MerkleProof<F>, params:&PoseidonParams<F>) -> Num<F> {
    assert!(proof.sibling.len() == proof.path.len(), "merkle proof path should be the same");
    let mut root = leaf.clone();
    
    for (&p, &s) in proof.path.iter().zip(proof.sibling.iter()) {
        let pair = if p {[s, root]} else {[root, s]};
        root = poseidon(pair.as_ref(), params);
    }
    root
}

pub fn poseidon_merkle_tree_root<F:PrimeField>(leaf:&[Num<F>], params: &PoseidonParams<F>) -> Num<F> {
    let leaf_sz = leaf.len();
    assert!(leaf_sz>0, "should be at least one leaf in the tree");
    let proof_sz = std::mem::size_of::<usize>() * 8 - (leaf_sz-1).leading_zeros() as usize;
    let total_leaf_sz = 1usize << proof_sz;
    let mut state = leaf.to_vec();
    state.extend_from_slice(&vec![Num::zero(); total_leaf_sz-leaf_sz]);
    for j in 0..proof_sz {
        for i in 0..total_leaf_sz>>(j + 1) {
            state[i] = poseidon(&[state[2*i].clone(), state[2*i+1].clone()], params);
        }
    }
    state[0].clone()
}
