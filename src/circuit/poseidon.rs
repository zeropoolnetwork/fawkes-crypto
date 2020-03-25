use bellman::{
    SynthesisError,
    ConstraintSystem
};

use bellman::pairing::{
    Engine
};


use super::signal::Signal;
use crate::poseidon::PoseidonParams;
use crate::wrappedmath::Wrap;


fn ark<E:Engine>(state: &mut[Signal<E>], c:Wrap<E::Fr>) {
    state.iter_mut().for_each(|e| *e = e.clone() + &Signal::Constant(c));
}

fn sigma<E:Engine, CS:ConstraintSystem<E>>(mut cs:CS, a:&Signal<E>) -> Result<Signal<E>, SynthesisError> {
    let a_sq = a.square(cs.namespace(|| "a^2"))?;
    let a_quad = a_sq.square(cs.namespace(|| "a^4"))?;
    a_quad.multiply(cs.namespace(|| "a^5"), a)
}

fn mix<E:Engine>(state: &mut[Signal<E>], params:&PoseidonParams<E::Fr>) {
    let statelen = state.len();
    let mut new_state = vec![Signal::zero(); statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] = &new_state[i] + params.m[i][j] * &state[j];
        }
    }
    state.clone_from_slice(&new_state);
}


pub fn poseidon<E:Engine, CS:ConstraintSystem<E>>(mut cs:CS, inputs:&[Signal<E>], params:&PoseidonParams<E::Fr>) -> Result<Signal<E>, SynthesisError> {
    let mut state = vec![Signal::zero(); params.t];
    let n_inputs = inputs.len();
    assert!(n_inputs <= params.t, "number of inputs should be less or equal than t");
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
    (&mut state[0..n_inputs]).clone_from_slice(inputs);

    let half_f = params.f>>1;

    for i in 0..params.f+params.p {
        ark(&mut state, params.c[i]);
        if i < half_f || i >= half_f + params.p {
            for j in 0..params.t {
                state[j] = sigma(cs.namespace(|| format!("sigma[{},{}]", i, j)), &state[j])?;
            }
        } else {
            state[0] = sigma(cs.namespace(|| format!("sigma[{}]", i)), &state[0])?;
        }
        mix(&mut state, params);
    }
    Ok(state[0].clone())
}


pub fn poseidon_merkle_root<E:Engine, CS:ConstraintSystem<E>>(
    mut cs:CS, leaf:&Signal<E>, 
    sibling:&[Signal<E>], 
    path:&[Signal<E>], 
    params:&PoseidonParams<E::Fr>
) -> Result<Signal<E>, SynthesisError> {
    assert!(sibling.len() == path.len(), "merkle proof length should be the same");
    let mut root = leaf.clone();
    let mut i = 0;
    for (p, s) in path.iter().zip(sibling.iter()) {
        i+=1;
        let first = s.switch(cs.namespace(|| format!("sel[{}]", i)), p, &root)?; 
        let second = &root + s - &first;
        root = poseidon(cs.namespace(|| format!("r[{}]", i)), [first, second].as_ref(), params)?;
    }
    Ok(root)
}


#[cfg(test)]
mod poseidon_test {
    use super::*;
    use sapling_crypto::circuit::test::TestConstraintSystem;
    use bellman::pairing::bn256::{Bn256, Fr};
    use rand::{Rng, thread_rng};

    #[test]
    fn test_circuit_poseidon() {
        const N_INPUTS: usize = 3;
        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(N_INPUTS+1, 8, 54);

    
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let data = (0..N_INPUTS).map(|_| rng.gen()).collect::<Vec<_>>();
        let inputs = (0..N_INPUTS).map(|i| Signal::alloc(cs.namespace(|| format!("{}th poseidon input", i)), Some(data[i])).unwrap()).collect::<Vec<_>>();
        
        let mut n_constraints = cs.num_constraints();
        let res = poseidon(cs.namespace(|| "compute poseidon"), &inputs, &poseidon_params).unwrap();
        n_constraints=cs.num_constraints()-n_constraints;
        
        let res2 = crate::poseidon::poseidon(&data, &poseidon_params);
        res.assert_constant(cs.namespace(|| "check res"), res2).unwrap();

        
        println!("poseidon(4,8,54) constraints = {}", n_constraints);
        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
        assert!(res.get_value().unwrap() == res2);
    }

    #[test]
    fn test_circuit_poseidon_merkle_root() {
        const PROOF_LENGTH: usize = 32;

        let mut rng = thread_rng();
        let poseidon_params = PoseidonParams::<Fr>::new(3, 8, 53);

    
        let mut cs = TestConstraintSystem::<Bn256>::new();



        let leaf = rng.gen();
        let sibling = (0..PROOF_LENGTH).map(|_| rng.gen()).collect::<Vec<_>>();
        let path = (0..PROOF_LENGTH).map(|_| rng.gen()).collect::<Vec<bool>>();

        let signal_leaf = Signal::alloc(cs.namespace(|| "leaf"), Some(leaf)).unwrap();
        let signal_sibling = (0..PROOF_LENGTH).map(|i| Signal::alloc(cs.namespace(|| format!("sibling[{}]", i)), Some(sibling[i])).unwrap()).collect::<Vec<_>>();
        let signal_path = (0..PROOF_LENGTH).map(|i| Signal::alloc(cs.namespace(|| format!("path[{}]", i)), Some(Wrap::from(path[i]))).unwrap()).collect::<Vec<_>>();
    
        
        
        let mut n_constraints = cs.num_constraints();
        let res = poseidon_merkle_root(cs.namespace(|| "merkle_root"), &signal_leaf, &signal_sibling, &signal_path, &poseidon_params).unwrap();
        n_constraints=cs.num_constraints()-n_constraints;
        
        let res2 = crate::poseidon::poseidon_merkle_root(leaf, &sibling, &path, &poseidon_params);
        res.assert_constant(cs.namespace(|| "check res"), res2).unwrap();

        
        println!("merkle root poseidon(3,8,53)x32 constraints = {}", n_constraints);
        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
        assert!(res.get_value().unwrap() == res2);
    }

}

