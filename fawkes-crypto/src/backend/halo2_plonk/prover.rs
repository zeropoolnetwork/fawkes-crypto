use halo2_curves::bn256::{Bn256, Fr, G1Affine};
use halo2_proofs::{
    dev::MockProver,
    plonk::{create_proof, Circuit, ProvingKey},
    poly::{
        commitment::Params,
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::ProverGWC,
        },
    },
    transcript::{EncodedChallenge, TranscriptWriterBuffer},
};
use itertools::Itertools;
use rand::rngs::OsRng;

pub fn gen_proof<
    C: Circuit<Fr>,
    E: EncodedChallenge<G1Affine>,
    TW: TranscriptWriterBuffer<Vec<u8>, G1Affine, E>,
>(
    params: &ParamsKZG<Bn256>,
    pk: &ProvingKey<G1Affine>,
    circuit: C,
    instances: Vec<Vec<Fr>>,
) -> Vec<u8> {
    MockProver::run(params.k(), &circuit, instances.clone())
        .unwrap()
        .assert_satisfied();

    let instances = instances
        .iter()
        .map(|instances| instances.as_slice())
        .collect_vec();
    let proof = {
        let mut transcript = TW::init(Vec::new());
        create_proof::<KZGCommitmentScheme<Bn256>, ProverGWC<_>, _, _, TW, _>(
            params,
            pk,
            &[circuit],
            &[instances.as_slice()],
            OsRng,
            &mut transcript,
        )
        .unwrap();
        transcript.finalize()
    };

    // let accept = {
    //     let mut transcript = TR::init(Cursor::new(proof.clone()));
    //     VerificationStrategy::<_, VerifierGWC<_>>::finalize(
    //         verify_proof::<_, VerifierGWC<_>, _, TR, _>(
    //             params.verifier_params(),
    //             pk.get_vk(),
    //             AccumulatorStrategy::new(params.verifier_params()),
    //             &[instances.as_slice()],
    //             &mut transcript,
    //         )
    //         .unwrap(),
    //     )
    // };

    proof
}
