use halo2_curves::bn256::{Bn256, Fr, G1Affine};
use halo2_proofs::{
    dev::MockProver,
    plonk::{create_proof, keygen_pk, keygen_vk, Circuit, ProvingKey},
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
    input_pub: Vec<Vec<Fr>>,
) -> Vec<u8> {
    MockProver::run(params.k(), &circuit, input_pub.clone())
        .unwrap()
        .assert_satisfied();

    let instances = input_pub
        .iter()
        .map(|input_pub| input_pub.as_slice())
        .collect_vec();
    let mut transcript = TW::init(Vec::new());
    create_proof::<KZGCommitmentScheme<Bn256>, ProverGWC<_>, _, _, TW, _>(
        params,
        &gen_pk(params, &circuit),
        &[circuit],
        &[instances.as_slice()],
        OsRng,
        &mut transcript,
    )
    .unwrap();
    transcript.finalize()
}
