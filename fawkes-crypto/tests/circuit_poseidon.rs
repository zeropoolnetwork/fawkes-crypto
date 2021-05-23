
use fawkes_crypto::{
    circuit::{cs::{DebugCS, CS}, poseidon::*, num::CNum},
    core::{signal::Signal, sizedvec::SizedVec},
    engines::bn256::Fr,
    native::poseidon::{poseidon, poseidon_merkle_proof_root, MerkleProof, PoseidonParams},
    rand::{thread_rng, Rng},
};
use std::time::Instant;

#[test]
fn test_circuit_poseidon() {
    const N_INPUTS: usize = 3;

    let mut rng = thread_rng();
    let poseidon_params = PoseidonParams::<Fr>::new(N_INPUTS + 1, 8, 54);

    let ref mut cs = DebugCS::rc_new();

    let data = (0..N_INPUTS)
        .map(|_| rng.gen())
        .collect::<SizedVec<_, N_INPUTS>>();
    let inputs = SizedVec::alloc(cs, Some(&data));

    let mut n_constraints = cs.borrow().num_gates();
    let res = c_poseidon(inputs.as_slice(), &poseidon_params);
    n_constraints = cs.borrow().num_gates() - n_constraints;

    let res2 = poseidon(data.as_slice(), &poseidon_params);
    res.assert_const(&res2);

    println!("poseidon(4,8,54) constraints = {}", n_constraints);
    assert!(res.get_value().unwrap() == res2);
}

#[test]
fn test_circuit_poseidon_merkle_root() {
    const PROOF_LENGTH: usize = 32;

    let mut rng = thread_rng();
    let poseidon_params = PoseidonParams::<Fr>::new(3, 8, 53);

    let ref mut cs = DebugCS::rc_new();

    let leaf = rng.gen();
    let sibling = (0..PROOF_LENGTH)
        .map(|_| rng.gen())
        .collect::<SizedVec<_, PROOF_LENGTH>>();
    let path = (0..PROOF_LENGTH)
        .map(|_| rng.gen())
        .collect::<SizedVec<bool, PROOF_LENGTH>>();

    let signal_leaf = CNum::alloc(cs, Some(&leaf));
    let signal_sibling = SizedVec::alloc(cs, Some(&sibling));
    let signal_path = SizedVec::alloc(cs, Some(&path));

    let mut n_constraints = cs.borrow().num_gates();
    let ref signal_proof = CMerkleProof {
        sibling: signal_sibling,
        path: signal_path,
    };
    let now = Instant::now();
    let res = c_poseidon_merkle_proof_root(&signal_leaf, &signal_proof, &poseidon_params);
    let elapsed = now.elapsed();

    n_constraints = cs.borrow().num_gates() - n_constraints;

    let proof = MerkleProof { sibling, path };
    let res2 = poseidon_merkle_proof_root(leaf, &proof, &poseidon_params);
    res.assert_const(&res2);

    println!(
        "merkle root poseidon(3,8,53)x32 constraints = {}",
        n_constraints
    );
    println!("circuit constructing time = {} ms", elapsed.as_millis());
    assert!(res.get_value().unwrap() == res2);
}
