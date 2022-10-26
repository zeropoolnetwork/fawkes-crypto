use halo2_curves::bn256::{Bn256, Fr, G1Affine};
// use super::group::G1Point as G1Affine;
// use super::engines::Bn256;
// use crate::engines::bn256::Fr;
use halo2_proofs::{
    dev::MockProver,
    plonk::{create_proof, Circuit,  keygen_pk, keygen_vk, ProvingKey},
    poly::{
        commitment::Params,
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::ProverGWC,
        },
    },
    transcript::{EncodedChallenge, TranscriptWriterBuffer},
};
// use itertools::Itertools;
use rand::rngs::OsRng;

fn gen_pk<C: Circuit<Fr>>(params: &ParamsKZG<Bn256>, circuit: &C) -> ProvingKey<G1Affine> {
    keygen_pk(params, keygen_vk(params, circuit).unwrap(), circuit).unwrap()
}

pub fn prove<
    C: Circuit<Fr>,
    E: EncodedChallenge<G1Affine>,
    TW: TranscriptWriterBuffer<Vec<u8>, G1Affine, E>,
>(
    params: &ParamsKZG<Bn256>,
    circuit: C,
) -> Vec<u8> {
    MockProver::run(params.k(), &circuit, vec![])
        .unwrap()
        .assert_satisfied();
    let proof = {
        let mut transcript = TW::init(Vec::new());
        create_proof::<KZGCommitmentScheme<Bn256>, ProverGWC<_>, _, _, TW, _>(
            params,
            &gen_pk(params, &circuit),
            &[circuit],
            &[&[]],
            OsRng,
            &mut transcript,
        )
        .unwrap();
        transcript.finalize()
    };

    proof
}
