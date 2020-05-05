use crate::core::cs::{Circuit};
use crate::core::osrng::OsRng;
use crate::core::field::Field;


use bellman::{self, SynthesisError};

use pairing::bn256::{Fq, Bn256, G1Affine, G2Affine};

use pairing::{
    Engine
};


use crate::helpers::groth16::Groth16CS;
use super::{G1PointData, G2PointData};

pub use bellman::groth16::Parameters;

pub struct Proof<E:Engine>(pub bellman::groth16::Proof<E>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize="", deserialize=""))]
pub struct ProofData<F:Field> {
    pub a: G1PointData<F>,
    pub b: G2PointData<F>,
    pub c: G1PointData<F>
}

impl Proof<Bn256> {
    pub fn into_data(&self) -> ProofData<Fq> {
        ProofData {
            a: G1PointData::from(self.0.a),
            b: G2PointData::from(self.0.b),
            c: G1PointData::from(self.0.c)
        }
    }

    pub fn from_data(p:&ProofData<Fq>) -> Self {
        Self(bellman::groth16::Proof{
            a: Into::<G1Affine>::into(p.a),
            b: Into::<G2Affine>::into(p.b),
            c: Into::<G1Affine>::into(p.c)
        })
    }
}


struct HelperCircuit<'a, C:Circuit>(pub &'a C);

impl <'a, BE:bellman::pairing::Engine, C:Circuit<F=BE::Fr>> bellman::Circuit<BE> for HelperCircuit<'a, C> {
    fn synthesize<CS: bellman::ConstraintSystem<BE>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let ref cs = Groth16CS::new(cs.namespace(|| "root"));
        self.0.synthesize(cs);
        Ok(())
    }
}

pub fn generate_keys<BE:bellman::pairing::Engine, C:Circuit<F=BE::Fr>+Default>() -> bellman::groth16::Parameters<BE> {
    let ref mut rng = OsRng::new();
    let c = C::default();
    bellman::groth16::generate_random_parameters(HelperCircuit(&c), rng).unwrap()
}

pub fn prove<BE:bellman::pairing::Engine, C:Circuit<F=BE::Fr>>(c:&C, params:&bellman::groth16::Parameters<BE>) -> Proof<BE>{
    let ref mut rng = OsRng::new();
    Proof(bellman::groth16::create_random_proof(HelperCircuit(c), params, rng).unwrap())
}

#[cfg(test)]
mod bellman_test {
    use super::*;
    use crate::core::field::Field;
    use bellman::pairing::bn256::{Fr, Bn256};
    use crate::circuit::num::{CNum};
    use crate::core::signal::Signal;
    use crate::core::cs::{TestCS, ConstraintSystem};
    use crate::native::poseidon::PoseidonParams;
    use crate::native::num::Num;
    use crate::circuit::poseidon::c_poseidon;
    use crate::helpers::groth16::verifier::{truncate_verifying_key, verify};
    use rand::{Rng, thread_rng};

    #[derive(Default)]
    struct CheckPreimageKnowledge<F:Field> {
        image:Option<Num<F>>,
        preimage:Option<Num<F>>
    }

    impl<F:Field> Circuit for CheckPreimageKnowledge<F> {
        type F = F;
        fn synthesize<CS: ConstraintSystem<F=F>>(
            &self,
            cs: &CS
        ) {
            let image = CNum::alloc(cs, self.image.as_ref());
            image.inputize();
            let preimage = CNum::alloc(cs, self.preimage.as_ref());
            let ref poseidon_params = PoseidonParams::<F>::new(2, 8, 53);
            let image_computed = c_poseidon([preimage].as_ref(), poseidon_params);
            (&image-&image_computed).assert_zero();
        }

        fn get_inputs(&self) -> Option<Vec<Num<Self::F>>> {
            let ref cs = TestCS::new();
            let image =CNum::alloc(cs, self.image.as_ref());
            image.linearize().iter().map(|e| e.get_value()).collect()
        }
    }


    #[test]
    fn test_helper() {
        let mut rng = thread_rng();
        let params = generate_keys::<Bn256, CheckPreimageKnowledge<Fr>>();
        let preimage = rng.gen();
        let ref poseidon_params = PoseidonParams::<Fr>::new(2, 8, 53);
        let image = crate::native::poseidon::poseidon([preimage].as_ref(), poseidon_params);
        let c = CheckPreimageKnowledge {image:Some(image), preimage:Some(preimage)};
        let proof = prove(&c, &params);

        let pvk = truncate_verifying_key(&params.vk);
        let res = verify(&pvk, &proof, [image.into_inner()].as_ref()).unwrap();
        assert!(res, "proof should be valid");
    }


}
