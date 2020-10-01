use super::*;
use super::prover::Proof;

pub struct VK<E:Engine> {
    alpha:G1Point<E>,
    beta:G2Point<E>,
    gamma:G2Point<E>,
    delta:G2Point<E>,
    ic:Vec<G1Point<E>>
}


impl<E:Engine> VK<E> {
    // fill missing fields with zeroes
    pub fn to_bellman(&self) -> bellman::groth16::VerifyingKey<E::BE> {
        bellman::groth16::VerifyingKey {
            alpha_g1: self.alpha.to_bellman(),
            beta_g1: <E::BE as bellman::pairing::Engine>::G1Affine::zero(),
            beta_g2: self.beta.to_bellman(),
            gamma_g2: self.gamma.to_bellman(),
            delta_g1: <E::BE as bellman::pairing::Engine>::G1Affine::zero(),
            delta_g2: self.delta.to_bellman(),
            ic: self.ic.iter().map(|e| e.to_bellman()).collect()

        }
    }

    pub fn from_bellman(vk:&bellman::groth16::VerifyingKey<E::BE>) -> Self {
        Self {
            alpha: G1Point::from_bellman(&vk.alpha_g1),
            beta: G2Point::from_bellman(&vk.beta_g2),
            gamma: G2Point::from_bellman(&vk.gamma_g2),
            delta: G2Point::from_bellman(&vk.delta_g2),
            ic: vk.ic.iter().map(|e| G1Point::from_bellman(e)).collect()
        }
    }
}

pub fn verify<E:Engine>(vk:&VK<E>, proof:&Proof<E>, inputs:&[Num<E::Fr>]) -> bool {
    let inputs:Vec<_> = inputs.iter().map(|e| num_to_bellman_fp(*e)).collect();
    let vk = vk.to_bellman();
    let proof = proof.to_bellman();
    let pvk = bellman::groth16::prepare_verifying_key(&vk);
    bellman::groth16::verify_proof(&pvk, &proof, &inputs).unwrap()
}