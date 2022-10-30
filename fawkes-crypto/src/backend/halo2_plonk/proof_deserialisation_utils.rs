use halo2_curves::bn256::{Fr, G1Affine};
use halo2_proofs::{
    plonk::{Error, VerifyingKey},
    // poly::commitment::{CommitmentScheme, Verifier},
    transcript::{EncodedChallenge, TranscriptRead},
};

pub struct LookupPermutationCommitments {
    pub permuted_input_commitment: G1Affine,
    pub permuted_table_commitment: G1Affine,
}

pub struct LookupCommitted {
    pub permuted: LookupPermutationCommitments,
    pub product_commitment: G1Affine,
}

pub struct CommonEvaluated {
    permutation_evals: Vec<Fr>,
}

pub struct Constructed {
    pub h_commitments: Vec<G1Affine>,
    pub random_poly_commitment: G1Affine,
}

pub struct PartiallyEvaluated {
    pub h_commitments: Vec<G1Affine>,
    pub random_poly_commitment: G1Affine,
    pub random_eval: Fr,
}

pub struct LookupEvaluated {
    pub committed: LookupCommitted,
    pub product_eval: Fr,
    pub product_next_eval: Fr,
    pub permuted_input_eval: Fr,
    pub permuted_input_inv_eval: Fr,
    pub permuted_table_eval: Fr,
}

pub struct PermutationEvaluated {
    sets: Vec<EvaluatedSet>,
}

pub struct EvaluatedSet {
    permutation_product_commitment: G1Affine,
    permutation_product_eval: Fr,
    permutation_product_next_eval: Fr,
    permutation_product_last_eval: Option<Fr>,
}

pub struct CommittedPermutations {
    pub permutation_product_commitments: Vec<G1Affine>,
}

pub fn read_permuted_commitments<
    'params,
    E: EncodedChallenge<G1Affine>,
    T: TranscriptRead<G1Affine, E>,
>(
    transcript: &mut T,
) -> Result<LookupPermutationCommitments, Error> {
    let permuted_input_commitment = transcript.read_point()?;
    let permuted_table_commitment = transcript.read_point()?;

    Ok(LookupPermutationCommitments {
        permuted_input_commitment,
        permuted_table_commitment,
    })
}

pub fn vk_read_product_commitments<
    'params,
    E: EncodedChallenge<G1Affine>,
    T: TranscriptRead<G1Affine, E>,
>(
    vk: &VerifyingKey<G1Affine>,
    transcript: &mut T,
) -> Result<CommittedPermutations, Error> {
    let chunk_len = vk.cs().degree() - 2;
    let permutation = vk.cs().permutation();

    let permutation_product_commitments = permutation
        .get_columns()
        .chunks(chunk_len)
        .map(|_| transcript.read_point())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CommittedPermutations {
        permutation_product_commitments,
    })
}

pub fn lookup_read_product_commitment<
    E: EncodedChallenge<G1Affine>,
    T: TranscriptRead<G1Affine, E>,
>(
    lookups: LookupPermutationCommitments,
    transcript: &mut T,
) -> Result<LookupCommitted, Error> {
    let product_commitment = transcript.read_point()?;

    Ok(LookupCommitted {
        permuted: lookups,
        product_commitment,
    })
}

pub fn permutation_evaluate<E: EncodedChallenge<G1Affine>, T: TranscriptRead<G1Affine, E>>(
    commitments_vec: &Vec<G1Affine>,
    transcript: &mut T,
) -> Result<CommonEvaluated, Error> {
    let permutation_evals = commitments_vec
        .iter()
        .map(|_| transcript.read_scalar())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CommonEvaluated { permutation_evals })
}

pub fn permutation_committed_evaluate<
    E: EncodedChallenge<G1Affine>,
    T: TranscriptRead<G1Affine, E>,
>(
    permutation_product_commitments: Vec<G1Affine>,
    transcript: &mut T,
) -> Result<PermutationEvaluated, Error> {
    let mut sets = vec![];

    let mut iter = permutation_product_commitments.into_iter();

    while let Some(permutation_product_commitment) = iter.next() {
        let permutation_product_eval = transcript.read_scalar()?;
        let permutation_product_next_eval = transcript.read_scalar()?;
        let permutation_product_last_eval = if iter.len() > 0 {
            Some(transcript.read_scalar()?)
        } else {
            None
        };

        sets.push(EvaluatedSet {
            permutation_product_commitment,
            permutation_product_eval,
            permutation_product_next_eval,
            permutation_product_last_eval,
        });
    }

    Ok(PermutationEvaluated { sets })
}

pub fn lookup_evaluate<E: EncodedChallenge<G1Affine>, T: TranscriptRead<G1Affine, E>>(
    lookup_committed: LookupCommitted,
    transcript: &mut T,
) -> Result<LookupEvaluated, Error> {
    let product_eval = transcript.read_scalar()?;
    let product_next_eval = transcript.read_scalar()?;
    let permuted_input_eval = transcript.read_scalar()?;
    let permuted_input_inv_eval = transcript.read_scalar()?;
    let permuted_table_eval = transcript.read_scalar()?;

    Ok(LookupEvaluated {
        committed: lookup_committed,
        product_eval,
        product_next_eval,
        permuted_input_eval,
        permuted_input_inv_eval,
        permuted_table_eval,
    })
}
