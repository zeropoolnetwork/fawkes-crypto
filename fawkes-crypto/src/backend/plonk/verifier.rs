use super::*;

use halo2_proofs::{
    plonk::verify_proof,
    poly::{
        commitment::ParamsProver,
        kzg::{
            multiopen::{VerifierGWC},
            strategy::AccumulatorStrategy,
        },
        VerificationStrategy,
    },
    transcript::{TranscriptReadBuffer}
};

use plonk_verifier::system::halo2::transcript::evm::EvmTranscript;
use itertools::Itertools;
use std::io::Cursor;
use super::setup::VK;
use super::prover::Proof;


pub fn verify(
    params: &Parameters<super::engines::Bn256>,
    vk: &VK<super::engines::Bn256>,
    proof: &Proof,
    inputs: &[Num<crate::engines::bn256::Fr>]
) -> bool {
    let instances = inputs.iter().cloned().map(num_to_halo_fp).collect_vec();

    let mut transcript =
        <EvmTranscript::<halo2_curves::bn256::G1Affine, _, _, _> as TranscriptReadBuffer<_, _, _>>
        ::init(Cursor::new(proof.0.clone()));
    VerificationStrategy::<_, VerifierGWC<_>>::finalize(
        verify_proof::<_, VerifierGWC<_>, _, EvmTranscript::<halo2_curves::bn256::G1Affine, _, _, _>, _>(
            params.0.verifier_params(),
            &vk.0,
            AccumulatorStrategy::new(params.0.verifier_params()),
            &[&[instances.as_slice()]],
            &mut transcript,
        ).unwrap(),
    )

}
