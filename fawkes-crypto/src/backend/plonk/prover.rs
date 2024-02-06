
use super::*;

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "borsh_support")]
use borsh::{BorshSerialize, BorshDeserialize};

use halo2_proofs::{
    plonk::{
        create_proof,Circuit, ProvingKey as PlonkProvingKey,
    },
    poly::{
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverGWC},
        },
    },
    transcript::{EncodedChallenge, TranscriptReadBuffer, TranscriptWriterBuffer}
};

use plonk_verifier::system::halo2::transcript::evm::EvmTranscript;
use itertools::Itertools;
use std::{
    io::{Cursor},
    rc::Rc,
    cell::{RefCell}
};

use halo2_curves::pairing::{MultiMillerLoop};

use halo2_rand::rngs::OsRng;

use crate::circuit::cs::BuildCS;

use super::setup::{ProvingKey};

#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "", deserialize = "")))]
#[derive(Clone, Debug, BorshDeserialize, BorshSerialize)]
pub struct Proof(pub Vec<u8>);

pub fn prove<
    Pub: Signal<BuildCS<crate::engines::bn256::Fr>>,
    Sec: Signal<BuildCS<crate::engines::bn256::Fr>>,
    C: Fn(Pub, Sec)
>(
    params: &Parameters<super::engines::Bn256>,
    pk: &ProvingKey<super::engines::Bn256>,
    input_pub: &Pub::Value,
    input_sec: &Sec::Value,
    circuit: C,
) -> (Vec<Num<crate::engines::bn256::Fr>>, Proof)
{
    let cs = BuildCS::<crate::engines::bn256::Fr>::new(false);
    let ref rcs = Rc::new(RefCell::new(cs));

    let signal_pub = Pub::alloc(rcs, Some(input_pub));
    signal_pub.inputize();
    let signal_sec = Sec::alloc(rcs, Some(input_sec));

    circuit(signal_pub, signal_sec);

    let bcs = HaloCS::<BuildCS<crate::engines::bn256::Fr>>::new(rcs.clone());

    let inputs = {
        let cs = rcs.borrow();
        let mut res = Vec::with_capacity(cs.num_input());
        for i in 0..cs.num_input() {
            res.push(cs.get_value(cs.as_public()[i]).unwrap())
        }
        res
    };

    let inputs_converted = inputs.iter().cloned().map(num_to_halo_fp).collect_vec();

    let proof = {
        let params = &params.0;
        let pk = &pk.0;
        let instances = vec![inputs_converted];
        // MockProver::run(params.k(), &circuit, instances.clone())
        //     .unwrap()
        //     .assert_satisfied();

        let instances = instances
            .iter()
            .map(|instances| instances.as_slice())
            .collect_vec();

        let mut transcript = <EvmTranscript<halo2_curves::bn256::G1Affine, _, _, _> as TranscriptWriterBuffer<_, _, _>>::init(Vec::new());
        create_proof::<KZGCommitmentScheme<halo2_curves::bn256::Bn256>, ProverGWC<_>, _, _, EvmTranscript<halo2_curves::bn256::G1Affine, _, _, _>, _>(
            params,
            pk,
            &[bcs],
            &[instances.as_slice()],
            OsRng,
            &mut transcript,
        )
        .unwrap();
        transcript.finalize()
    };

    (inputs, Proof(proof))
}
