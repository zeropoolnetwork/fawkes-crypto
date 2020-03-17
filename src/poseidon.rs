use bellman_ce::pairing::ff::{
    Field,
    PrimeField
};

use rand::Rng;

use crate::seedbox::SeedboxBlake2;

pub struct PoseidonParams<F:Field> {
    pub c: Vec<F>, 
    pub m: Vec<Vec<F>>, 
    pub t: usize, 
    pub f: usize, 
    pub p: usize
}

impl<F:PrimeField> PoseidonParams<F> {
    pub fn new(t:usize, f:usize, p:usize) -> Self {
        let mut seedbox = SeedboxBlake2::new(format!("fawkes_poseidon(t={},f={},p={})", t, f, p).as_bytes());
        let c = (0..f+p).map(|_| seedbox.gen()).collect();
        let m = (0..t).map(|_| (0..t).map(|_| seedbox.gen()).collect()).collect();
        PoseidonParams {c, m, t, f, p}
    }
}




fn ark<F:PrimeField>(state: &mut[F], c:&F) {
    state.iter_mut().for_each(|e| e.add_assign(c))
}

fn sigma<F:PrimeField>(a: &F) -> F {
    let mut res = a.clone();
    res.square();
    res.square();
    res.mul_assign(a);
    res
}

fn mix<F:PrimeField>(state: &mut[F], params:&PoseidonParams<F>) {
    let statelen = state.len();
    let mut new_state = vec![F::zero(); statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            let mut m = params.m[i][j];
            m.mul_assign(&state[j]);
            new_state[i].add_assign(&m);
        }
    }
    state.clone_from_slice(&new_state);
}



pub fn poseidon<F:PrimeField>(inputs:&[F], params:&PoseidonParams<F>) -> F {
    let mut state = vec![F::zero(); params.t];
    let n_inputs = inputs.len();
    assert!(n_inputs <= params.t, "number of inputs should be less or equal than t");
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");
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
    state[0]
}


pub fn merkle_root<F:PrimeField>(leaf:&F, siblings:&[F], path:&[bool], params:&PoseidonParams<F>) -> F {
    assert!(siblings.len() == path.len(), "merkle proof path should be the same");
    let mut root = leaf.clone();
    
    for (&p, &s) in path.iter().zip(siblings.iter()) {
        let pair = if p {[s, root]} else {[root, s]};
        root = poseidon(pair.as_ref(), params);
    }
    root
}
