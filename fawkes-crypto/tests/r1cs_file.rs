use fawkes_crypto::{
    backend::bellman_groth16::{
        engines::Bn256,
        setup::setup
    },
    circuit::cs::CS,
    circuit::num::CNum,
    circuit::poseidon::{c_poseidon_merkle_proof_root, CMerkleProof},
    core::signal::Signal,
    native::poseidon::{PoseidonParams},
};
use fawkes_crypto::backend::r1cs::get_r1cs_file;
use fawkes_crypto::backend::bellman_groth16::engines::Engine;

#[cfg(feature = "r1cs-file")]
#[test]
fn test_parameters_get_r1cs_file() {
    fn circuit<C:CS>(public: CNum<C>, secret: (CNum<C>, CMerkleProof<C, 32>)) {
        let poseidon_params = PoseidonParams::<C::Fr>::new(3, 8, 53);
        let res = c_poseidon_merkle_proof_root(&secret.0, &secret.1, &poseidon_params);
        res.assert_eq(&public);
    }

    let params = setup::<Bn256, _, _, _>(circuit);

    let file = get_r1cs_file::<<Bn256 as Engine>::Fr, 32>(&params.1);

    assert!(true)
}