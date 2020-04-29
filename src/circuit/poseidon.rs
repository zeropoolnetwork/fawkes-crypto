use typenum::Unsigned;

use crate::circuit::num::{CNum};
use crate::circuit::bool::{CBool};
use crate::core::signal::Signal;
use crate::core::sizedvec::SizedVec;
use crate::core::cs::ConstraintSystem;
use crate::native::poseidon::{PoseidonParams, MerkleProof};
use crate::native::num::Num;


#[derive(Clone, Signal)]
#[Value="MerkleProof<CS::F, L>"]
pub struct CMerkleProof<'a, CS:ConstraintSystem, L:Unsigned> {
    pub sibling: SizedVec<CNum<'a, CS>, L>,
    pub path: SizedVec<CBool<'a, CS>, L>
}

fn ark<'a, CS:ConstraintSystem>(state: &mut[CNum<'a, CS>], c:Num<CS::F>) {
    state.iter_mut().for_each(|e| *e += c);
}

fn sigma<'a, CS:ConstraintSystem>(a:&CNum<'a, CS>) -> CNum<'a, CS> {
    let a_sq = a*a;
    let a_quad = &a_sq*&a_sq;
    a_quad*a
}

fn mix<'a, CS:ConstraintSystem>(state: &mut[CNum<'a, CS>], params:&PoseidonParams<CS::F>) {
    let statelen = state.len();
    let cs = state[0].cs;
    let mut new_state = vec![CNum::zero(cs); statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.m[i][j] * &state[j];
        }
    }
    state.clone_from_slice(&new_state);
}


pub fn c_poseidon<'a, CS:ConstraintSystem>(inputs:&[CNum<'a, CS>], params:&PoseidonParams<CS::F>) -> CNum<'a, CS> {
    let n_inputs = inputs.len();
    assert!(n_inputs <= params.t, "number of inputs should be less or equal than t");
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    let cs = inputs[0].cs;
    let mut state = vec![CNum::zero(cs); params.t];
    (&mut state[0..n_inputs]).clone_from_slice(inputs);

    let half_f = params.f>>1;

    for i in 0..params.f+params.p {
        ark(&mut state, params.c[i]);
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


pub fn c_poseidon_merkle_proof_root<'a, CS:ConstraintSystem, L:Unsigned>(
    leaf:&CNum<'a, CS>, 
    proof:&CMerkleProof<'a, CS, L>,
    params:&PoseidonParams<CS::F>
) -> CNum<'a, CS> {
    let mut root = leaf.clone();
    for (p, s) in proof.path.iter().zip(proof.sibling.iter()) {
        let first = s.switch(p, &root); 
        let second = &root + s - &first;
        root = c_poseidon( [first, second].as_ref(), params);
    }
    root
}

pub fn c_poseidon_merkle_tree_root<'a, CS:ConstraintSystem>(leaf: &[CNum<'a, CS>], params: &PoseidonParams<CS::F>) -> CNum<'a, CS> {
    let leaf_sz = leaf.len();
    assert!(leaf_sz>0, "should be at least one leaf in the tree");
    let cs = leaf[0].cs;
    let proof_sz = std::mem::size_of::<usize>() * 8 - (leaf_sz-1).leading_zeros() as usize;
    let total_leaf_sz = 1usize << proof_sz;
    let mut state = leaf.to_vec();
    state.extend_from_slice(&vec![CNum::zero(cs); total_leaf_sz-leaf_sz]);
    for j in 0..proof_sz {
        for i in 0..total_leaf_sz>>(j + 1) {
            state[i] = c_poseidon(&[state[2*i].clone(), state[2*i+1].clone()], params);
        }
    }
    state[0].clone()
}



#[cfg(test)]
mod poseidon_test {
    use super::*;
    use crate::core::cs::TestCS;
    use crate::native::poseidon::{poseidon, poseidon_merkle_proof_root, MerkleProof};
    use crate::core::signal::Signal;
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};
    use typenum::{U3, U32};
    
    

    #[test]
    fn test_circuit_poseidon() {
        const N_INPUTS: usize = 3;
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(N_INPUTS+1, 8, 54);

    
        let ref mut cs = TestCS::<Fr>::new();
        let data = (0..N_INPUTS).map(|_| rng.gen()).collect::<SizedVec<_, U3>>();
        let inputs = SizedVec::alloc(cs, Some(&data));
        
        let mut n_constraints = cs.num_constraints();
        let res = c_poseidon(&inputs.0, &poseidon_params);
        n_constraints=cs.num_constraints()-n_constraints;
        
        let res2 = poseidon(&data.0, &poseidon_params);
        res.assert_const(res2);

        
        println!("poseidon(4,8,54) constraints = {}", n_constraints);
        assert!(res.get_value().unwrap() == res2);
    }

    #[test]
    fn test_circuit_poseidon_merkle_root() {
        const PROOF_LENGTH: usize = 32;

        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(3, 8, 53);

    
        let ref mut cs = TestCS::<Fr>::new();



        let leaf = rng.gen();
        let sibling = (0..PROOF_LENGTH).map(|_| rng.gen()).collect::<SizedVec<_, U32>>();
        let path = (0..PROOF_LENGTH).map(|_| rng.gen()).collect::<SizedVec<bool, U32>>();

        let signal_leaf = CNum::alloc(cs, Some(&leaf));
        let signal_sibling = SizedVec::alloc(cs, Some(&sibling));
        let signal_path = SizedVec::alloc(cs, Some(&path));
    
        
        
        let mut n_constraints = cs.num_constraints();
        let ref signal_proof = CMerkleProof {sibling:signal_sibling, path:signal_path};
        let res = c_poseidon_merkle_proof_root(&signal_leaf, &signal_proof, &poseidon_params);
        n_constraints=cs.num_constraints()-n_constraints;
        
        let proof = MerkleProof {sibling, path};
        let res2 = poseidon_merkle_proof_root(leaf, &proof, &poseidon_params);
        res.assert_const(res2);

        println!("merkle root poseidon(3,8,53)x32 constraints = {}", n_constraints);
        assert!(res.get_value().unwrap() == res2);
    }

}
