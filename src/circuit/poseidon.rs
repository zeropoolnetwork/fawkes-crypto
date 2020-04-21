

use crate::core::signal::Signal;
use crate::core::num::Num;
use crate::core::cs::ConstraintSystem;
use crate::native::poseidon::{PoseidonParams};



fn ark<'a, CS:ConstraintSystem>(state: &mut[Signal<'a, CS>], c:Num<CS::F>) {
    state.iter_mut().for_each(|e| *e += c);
}

fn sigma<'a, CS:ConstraintSystem>(a:&Signal<'a, CS>) -> Signal<'a, CS> {
    let a_sq = a*a;
    let a_quad = &a_sq*&a_sq;
    a_quad*a
}

fn mix<'a, CS:ConstraintSystem>(state: &mut[Signal<'a, CS>], params:&PoseidonParams<CS::F>) {
    let statelen = state.len();
    let cs = state[0].cs;
    let mut new_state = vec![Signal::zero(cs); statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.m[i][j] * &state[j];
        }
    }
    state.clone_from_slice(&new_state);
}


pub fn c_poseidon<'a, CS:ConstraintSystem>(inputs:&[Signal<'a, CS>], params:&PoseidonParams<CS::F>) -> Signal<'a, CS> {
    let n_inputs = inputs.len();
    assert!(n_inputs <= params.t, "number of inputs should be less or equal than t");
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    let cs = inputs[0].cs;
    let mut state = vec![Signal::zero(cs); params.t];
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


pub fn c_poseidon_merkle_root<'a, CS:ConstraintSystem>(
    leaf:&Signal<'a, CS>, 
    sibling:&[Signal<'a, CS>], 
    path:&[Signal<'a, CS>], 
    params:&PoseidonParams<CS::F>
) -> Signal<'a, CS> {
    assert!(sibling.len() == path.len(), "merkle proof length should be the same");
    let mut root = leaf.clone();
    for (p, s) in path.iter().zip(sibling.iter()) {
        let first = s.switch(p, &root); 
        let second = &root + s - &first;
        root = c_poseidon( [first, second].as_ref(), params);
    }
    root
}

pub fn c_merkle_tree_root<'a, CS:ConstraintSystem>(leaf: &[Signal<'a, CS>], params: &PoseidonParams<CS::F>) -> Signal<'a, CS> {
    let leaf_sz = leaf.len();
    assert!(leaf_sz>0, "should be at least one leaf in the tree");
    let cs = leaf[0].cs;
    let proof_sz = std::mem::size_of::<usize>() * 8 - (leaf_sz-1).leading_zeros() as usize;
    let total_leaf_sz = 1usize << proof_sz;
    let mut state = leaf.to_vec();
    state.extend_from_slice(&vec![Signal::zero(cs); total_leaf_sz-leaf_sz]);
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
    use crate::native::poseidon::{poseidon, poseidon_merkle_root};
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};
    

    #[test]
    fn test_circuit_poseidon() {
        const N_INPUTS: usize = 3;
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(N_INPUTS+1, 8, 54);

    
        let ref mut cs = TestCS::<Fr>::new();
        let data = (0..N_INPUTS).map(|_| rng.gen()).collect::<Vec<_>>();
        let inputs = (0..N_INPUTS).map(|i| Signal::alloc(cs, Some(data[i]))).collect::<Vec<_>>();
        
        let mut n_constraints = cs.num_constraints();
        let res = c_poseidon(&inputs, &poseidon_params);
        n_constraints=cs.num_constraints()-n_constraints;
        
        let res2 = poseidon(&data, &poseidon_params);
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
        let sibling = (0..PROOF_LENGTH).map(|_| rng.gen()).collect::<Vec<_>>();
        let path = (0..PROOF_LENGTH).map(|_| rng.gen()).collect::<Vec<bool>>();

        let signal_leaf = Signal::alloc(cs, Some(leaf));
        let signal_sibling = (0..PROOF_LENGTH).map(|i| Signal::alloc(cs, Some(sibling[i]))).collect::<Vec<_>>();
        let signal_path = (0..PROOF_LENGTH).map(|i| Signal::alloc(cs, Some(Num::from(path[i])))).collect::<Vec<_>>();
    
        
        
        let mut n_constraints = cs.num_constraints();
        let res = c_poseidon_merkle_root(&signal_leaf, &signal_sibling, &signal_path, &poseidon_params);
        n_constraints=cs.num_constraints()-n_constraints;
        
        let res2 = poseidon_merkle_root(leaf, &sibling, &path, &poseidon_params);
        res.assert_const(res2);

        println!("merkle root poseidon(3,8,53)x32 constraints = {}", n_constraints);
        assert!(res.get_value().unwrap() == res2);
    }

}
