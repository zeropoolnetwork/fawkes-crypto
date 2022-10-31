use halo2_curves::bn256::{Bn256, Fq, Fr, G1Affine};
use halo2_proofs::{plonk::VerifyingKey, poly::{kzg::commitment::ParamsKZG, commitment::ParamsProver}};
use plonk_verifier::{
    loader::evm::EvmLoader,
    pcs::kzg::{Gwc19, Kzg, LimbsEncoding},
    system::halo2::{compile as compile_halo2, transcript::evm::EvmTranscript, Config},
    verifier::{self, PlonkProof, PlonkVerifier},
    Error,
};

use std::rc::Rc;

const LIMBS: usize = 4;
const BITS: usize = 68;

type Pcs = Kzg<Bn256, Gwc19>;
type Plonk = verifier::Plonk<Pcs, LimbsEncoding<LIMBS, BITS>>;

pub fn deserialize_kzg_proof(
    params: &ParamsKZG<Bn256>,
    vk: &VerifyingKey<G1Affine>,
    num_instance: Vec<usize>,
) -> Result<PlonkProof<G1Affine, Rc<EvmLoader>, Kzg<Bn256, Gwc19>>, Error> {
    let protocol = compile_halo2(
        params,
        vk,
        Config::kzg()
            .with_num_instance(num_instance.clone())
    );

    let svk = params.get_g()[0].into();
    let loader = EvmLoader::new::<Fq, Fr>();
    let mut transcript = EvmTranscript::<_, Rc<EvmLoader>, _, _>::new(loader.clone());

    let instances = transcript.load_instances(num_instance);
    Plonk::read_proof(&svk, &protocol, &instances, &mut transcript)
}
