use super::proof_deserialisation_utils::*;
use halo2_curves::bn256::{Fr, G1Affine};
use halo2_curves::CurveAffine;
use halo2_proofs::{
    plonk::{Error, VerifyingKey},
    transcript::{EncodedChallenge, TranscriptRead},
};
use std::io;

pub struct Proof {
    pub advice_commitments: Vec<Vec<G1Affine>>, // Plonk commitments
    pub advice_evals: Vec<Vec<Fr>>,             // Plonk evaluations
    pub challenges: Vec<Fr>,                    // Plonk commitments
    pub challenge_scalars: ChallengeScalars,
    pub fixed_evals: Vec<Fr>,                 // Plonk evaluations
    pub vanishing: PartiallyEvaluated,        // Vanishing argument
    pub permutations_common: CommonEvaluated, // Plonk evaluations
    pub permutations_evaluated: Vec<PermutationEvaluated>, // polynomial commitment scheme
    pub lookups_evaluated: Vec<Vec<LookupEvaluated>>, // Plonk commitments
}

pub fn deserialize_proof<'params, E: EncodedChallenge<G1Affine>, T: TranscriptRead<G1Affine, E>>(
    // params: &'params Scheme::ParamsVerifier,
    vk: &VerifyingKey<G1Affine>,
    instances: &[&[&[Fr]]],
    transcript: &mut T,
) -> Result<Proof, Error> {
    let num_proofs = instances.len();
    let (advice_commitments, challenges) = {
        let mut advice_commitments =
            vec![vec![G1Affine::default(); vk.cs().num_advice_columns()]; num_proofs];
        let mut challenges = vec![Fr::zero(); vk.cs().num_challenges()];

        for current_phase in 0..*vk.cs().advice_column_phase().iter().max().unwrap_or(&0u8) {
            for advice_commitments in advice_commitments.iter_mut() {
                for (phase, commitment) in vk
                    .cs()
                    .advice_column_phase()
                    .iter()
                    .zip(advice_commitments.iter_mut())
                {
                    if current_phase == *phase {
                        *commitment = transcript.read_point()?;
                    }
                }
            }
            for (phase, challenge) in vk.cs().challenge_phase().iter().zip(challenges.iter_mut()) {
                if current_phase == *phase {
                    *challenge = *transcript.squeeze_challenge_scalar::<()>();
                }
            }
        }

        (advice_commitments, challenges)
    };

    // Sample theta challenge for keeping lookup columns linearly independent
    let theta = transcript.squeeze_challenge_scalar::<()>();

    let lookups_permuted = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            // Hash each lookup permuted commitment
            vk.cs()
                .lookups()
                .iter()
                .map(|_| read_permuted_commitments(transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Sample beta challenge
    let beta = transcript.squeeze_challenge_scalar::<()>();

    // Sample gamma challenge
    let gamma = transcript.squeeze_challenge_scalar::<()>();

    let permutations_committed = (0..num_proofs)
        .map(|_| vk_read_product_commitments(vk, transcript))
        .collect::<Result<Vec<_>, _>>()?;

    let lookups_committed = lookups_permuted
        .into_iter()
        .map(|lookups| {
            // Hash each lookup product commitment
            lookups
                .into_iter()
                .map(|lookups| lookup_read_product_commitment(lookups, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let vanishing = transcript.read_point()?;

    let y = transcript.squeeze_challenge_scalar::<()>();

    let vanishing = Constructed {
        // before y
        random_poly_commitment: vanishing,
        // after y
        h_commitments: read_n_points(transcript, vk.get_domain().get_quotient_poly_degree())?,
    };
    // Sample x challenge, which is used to ensure the circuit is
    // satisfied with high probability.
    let x = transcript.squeeze_challenge_scalar::<()>();

    // instance_evals connected with transcript only if QUERY_INSTANCE is true on verifier
    // and after that those code
    // (0..num_proofs)
    // .map(|_| -> Result<Vec<_>, _> {
    //     read_n_scalars(transcript, vk.cs.instance_queries.len())
    // })
    // .collect::<Result<Vec<_>, _>>()?
    // however query_instance is true only on ipa, so not kzg case

    let advice_evals = (0..num_proofs)
        .map(|_| -> Result<Vec<_>, _> {
            read_n_scalars(transcript, vk.cs().advice_queries().len())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let fixed_evals = read_n_scalars(transcript, vk.cs().fixed_queries().len())?;

    let vanishing = PartiallyEvaluated {
        h_commitments: vanishing.h_commitments,
        random_poly_commitment: vanishing.random_poly_commitment,
        random_eval: transcript.read_scalar()?,
    };

    let permutations_common = permutation_evaluate(vk.permutation().commitments(), transcript)?;

    let permutations_evaluated = permutations_committed
        .into_iter()
        .map(|permutation| {
            permutation_committed_evaluate(permutation.permutation_product_commitments, transcript)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let lookups_evaluated = lookups_committed
        .into_iter()
        .map(|lookups| -> Result<Vec<_>, _> {
            lookups
                .into_iter()
                .map(|lookup| lookup_evaluate(lookup, transcript))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let v = transcript.squeeze_challenge_scalar();

    // let commitment_data = construct_intermediate_sets(queries);

    // let w: Vec<E::G1Affine> = (0..commitment_data.len())
    //     .map(|_| transcript.read_point().map_err(|_| Error::SamplingError))
    //     .collect::<Result<Vec<E::G1Affine>, Error>>()?;

    let u = transcript.squeeze_challenge_scalar();

    // in verify_proof, after all deserialization, it all chained into queries and put into verifier function,
    // where read some new points - last trouble it's the size of this commitment_data
    //         let commitment_data = construct_intermediate_sets(queries);
    // let w: Vec<E::G1Affine> = (0..commitment_data.len())
    // .map(|_| transcript.read_point().map_err(|_| Error::SamplingError))
    // .collect::<Result<Vec<E::G1Affine>, Error>>()?;

    let challenge_scalars = ChallengeScalars {
        theta,
        beta,
        gamma,
        y,
        x,
        v,
        u,
    };

    let proof = Proof {
        advice_commitments,
        advice_evals,
        challenges,
        challenge_scalars,
        fixed_evals,
        vanishing,
        permutations_common,
        permutations_evaluated,
        lookups_evaluated,
    };
    Ok(proof)
}

pub fn read_n_points<C: CurveAffine, E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C>> {
    (0..n).map(|_| transcript.read_point()).collect()
}

pub fn read_n_scalars<C: CurveAffine, E: EncodedChallenge<C>, T: TranscriptRead<C, E>>(
    transcript: &mut T,
    n: usize,
) -> io::Result<Vec<C::Scalar>> {
    (0..n).map(|_| transcript.read_scalar()).collect()
}
