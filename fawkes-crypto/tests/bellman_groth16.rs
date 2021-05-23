#[cfg(feature = "heavy_tests")]
use fawkes_crypto::{
    backend::bellman_groth16::{
        *,
        engines::Bn256,
        setup::setup
    },
    circuit::cs::CS,
    circuit::num::CNum,
    circuit::poseidon::{c_poseidon_merkle_proof_root, CMerkleProof},
    core::signal::Signal,
    core::sizedvec::SizedVec,
    engines::bn256::Fr,
    native::poseidon::{poseidon_merkle_proof_root, MerkleProof, PoseidonParams},
    rand::{thread_rng, Rng}
};

#[cfg(feature = "heavy_tests")]
#[test]
fn test_circuit_poseidon_merkle_root() {
    fn circuit<C:CS>(public: CNum<C>, secret: (CNum<C>, CMerkleProof<C, 32>)) {
        let poseidon_params = PoseidonParams::<C::Fr>::new(3, 8, 53);
        let res = c_poseidon_merkle_proof_root(&secret.0, &secret.1, &poseidon_params);
        res.assert_eq(&public);
    }
    let params = setup::<Bn256, _, _, _>(circuit);

    const PROOF_LENGTH: usize = 32;
    let mut rng = thread_rng();
    let poseidon_params = PoseidonParams::<Fr>::new(3, 8, 53);
    let leaf = rng.gen();
    let sibling = (0..PROOF_LENGTH)
        .map(|_| rng.gen())
        .collect::<SizedVec<_, 32>>();
    let path = (0..PROOF_LENGTH)
        .map(|_| rng.gen())
        .collect::<SizedVec<bool, 32>>();
    let proof = MerkleProof { sibling, path };
    let root = poseidon_merkle_proof_root(leaf, &proof, &poseidon_params);

    println!("BitVec length {}", params.2.len());

    let (inputs, snark_proof) = prover::prove(&params, &root, &(leaf, proof), circuit);

    let res = verifier::verify(&params.get_vk(), &snark_proof, &inputs);
    assert!(res, "Verifier result should be true");
}

