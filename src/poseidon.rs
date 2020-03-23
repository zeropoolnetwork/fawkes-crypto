use ff::{
    PrimeField
};

use rand::Rng;

use crate::seedbox::SeedboxBlake2;
use crate::wrappedmath::Wrap;

pub struct PoseidonParams<F:PrimeField> {
    pub c: Vec<Wrap<F>>, 
    pub m: Vec<Vec<Wrap<F>>>, 
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




fn ark<F:PrimeField>(state: &mut[Wrap<F>], c:Wrap<F>) {
    state.iter_mut().for_each(|e| *e += c)
}

fn sigma<F:PrimeField>(a: Wrap<F>) -> Wrap<F> {
    a.square().square()*a
}

fn mix<F:PrimeField>(state: &mut[Wrap<F>], params:&PoseidonParams<F>) {
    let statelen = state.len();
    let mut new_state = vec![Wrap::zero(); statelen];
    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.m[i][j] * state[j];
        }
    }
    state.clone_from_slice(&new_state);
}



pub fn poseidon<F:PrimeField>(inputs:&[Wrap<F>], params:&PoseidonParams<F>) -> Wrap<F> {
    let mut state = vec![Wrap::zero(); params.t];
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


pub fn merkle_root<F:PrimeField>(leaf:&Wrap<F>, siblings:&[Wrap<F>], path:&[bool], params:&PoseidonParams<F>) -> Wrap<F> {
    assert!(siblings.len() == path.len(), "merkle proof path should be the same");
    let mut root = leaf.clone();
    
    for (&p, &s) in path.iter().zip(siblings.iter()) {
        let pair = if p {[s, root]} else {[root, s]};
        root = poseidon(pair.as_ref(), params);
    }
    root
}
