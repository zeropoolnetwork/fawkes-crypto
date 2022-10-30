use super::proof_deserialisation_utils::*;
use halo2_curves::bn256::{Fr, G1Affine};
use halo2_curves::CurveAffine;
use halo2_proofs::{
    plonk::{Error, VerifyingKey},
    transcript::{EncodedChallenge, TranscriptRead},
};
use std::io;

pub struct Proof {
    pub advice_commitments: Vec<Vec<G1Affine>>,
    pub advice_evals: Vec<Vec<Fr>>,
    pub fixed_evals: Vec<Fr>,
    pub vanishing: PartiallyEvaluated,
    pub permutations_common: CommonEvaluated,
    pub permutations_evaluated: Vec<PermutationEvaluated>,
    pub lookups_evaluated: Vec<Vec<LookupEvaluated>>,
}

pub fn deserialize_proof<'params, E: EncodedChallenge<G1Affine>, T: TranscriptRead<G1Affine, E>>(
    // params: &'params Scheme::ParamsVerifier,
    vk: &VerifyingKey<G1Affine>,
    instances: &[&[&[Fr]]],
    transcript: &mut T,
) -> Result<Proof, Error> {
    let num_proofs = instances.len();
    let advice_commitments =
        vec![vec![transcript.read_point()?; vk.cs().num_advice_columns()]; num_proofs];

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

    let vanishing = Constructed {
        // before y
        random_poly_commitment: transcript.read_point()?,
        // after y
        h_commitments: read_n_points(transcript, vk.get_domain().get_quotient_poly_degree())?,
    };

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
    
    // in verify_proof, after all deserialization, it all chained into queries and put into verifier function, 
    // where read some new points - last trouble it's the size of this commitment_data
    //         let commitment_data = construct_intermediate_sets(queries);
    // let w: Vec<E::G1Affine> = (0..commitment_data.len())
    // .map(|_| transcript.read_point().map_err(|_| Error::SamplingError))
    // .collect::<Result<Vec<E::G1Affine>, Error>>()?;

    let proof = Proof {
        advice_commitments,
        advice_evals,
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
