use crate::core::cs::{Circuit};
use crate::core::osrng::OsRng;


use bellman::{self, SynthesisError};

use crate::helpers::groth16::Groth16CS;


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

pub fn proof<BE:bellman::pairing::Engine, C:Circuit<F=BE::Fr>>(c:&C, params:&bellman::groth16::Parameters<BE>) -> bellman::groth16::Proof<BE>{
    let ref mut rng = OsRng::new();
    bellman::groth16::create_random_proof(HelperCircuit(c), params, rng).unwrap()
}

#[cfg(test)]
mod bellman_test {
    use super::*;
    use ff::{PrimeField, SqrtField};
    use bellman::pairing::bn256::{Fr, Bn256};
    use crate::circuit::num::{CNum};
    use crate::core::signal::Signal;
    use crate::core::cs::{TestCS, ConstraintSystem};
    use crate::native::poseidon::PoseidonParams;
    use crate::native::num::Num;
    use crate::circuit::poseidon::c_poseidon;
    use rand::{Rng, thread_rng};

    #[derive(Default)]
    struct CheckPreimageKnowledge<F:PrimeField+SqrtField> {
        image:Option<Num<F>>,
        preimage:Option<Num<F>>
    }

    impl<F:PrimeField+SqrtField> Circuit for CheckPreimageKnowledge<F> {
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
        let proof = proof(&c, &params);

        let pvk = bellman::groth16::prepare_verifying_key(&params.vk);
        let res = bellman::groth16::verify_proof(&pvk, &proof, [image.into_inner()].as_ref()).unwrap();
        assert!(res, "proof should be valid");
    }


}
