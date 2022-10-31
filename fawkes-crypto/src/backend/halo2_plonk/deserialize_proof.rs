use halo2_curves::bn256::{Bn256, Fr, G1Affine};
use halo2_proofs::transcript::{Blake2bRead, Challenge255, TranscriptReadBuffer};
use halo2_proofs::{
    plonk::VerifyingKey,
    poly::{commitment::ParamsProver, kzg::commitment::ParamsKZG},
};
use plonk_verifier::{
    loader::native::NativeLoader,
    pcs::kzg::{Gwc19, Kzg, LimbsEncoding},
    system::halo2::{compile, Config},
    verifier::{self, PlonkProof, PlonkVerifier},
    Error,
};

const LIMBS: usize = 4;
const BITS: usize = 68;

type Pcs = Kzg<Bn256, Gwc19>;
type Plonk = verifier::Plonk<Pcs, LimbsEncoding<LIMBS, BITS>>;

pub fn deserialize_kzg_proof(
    params: &ParamsKZG<Bn256>,
    vk: &VerifyingKey<G1Affine>,
    instances: Vec<Vec<Fr>>,
    proof: &[u8],
) -> Result<PlonkProof<G1Affine, NativeLoader, Kzg<Bn256, Gwc19>>, Error> {
    let protocol = compile(params, vk, Config::kzg());

    let svk = params.get_g()[0].into();
    let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof);

    Plonk::read_proof(&svk, &protocol, &instances, &mut transcript)
}
