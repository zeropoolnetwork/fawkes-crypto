use fawkes_crypto::backend::halo2_plonk::{
    deserialize_proof::deserialize_kzg_proof, prover::prove,
};
use halo2_curves::bn256::{Bn256, Fr};
use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{keygen_vk, Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::{commitment::ParamsProver, kzg::commitment::ParamsKZG, Rotation},
    transcript::{Blake2bWrite, Challenge255},
};

use rand::rngs::OsRng;
use rand::RngCore;

#[test]
fn test_create_proof() {
    const K: u32 = 4;

    let circuit = StandardPlonk::rand(OsRng);

    let params = ParamsKZG::<Bn256>::new(K);

    // Create proof with correct number of instances
    let proof = prove::<_, _, Blake2bWrite<_, _, Challenge255<_>>>(
        &params,
        circuit.clone(),
        circuit.instances(),
    );

    let des_proof = deserialize_kzg_proof(
        &params,
        &keygen_vk(&params, &circuit).unwrap(),
        circuit.instances(),
        &proof,
    );

    println!("{:?}", des_proof);
    des_proof.unwrap();
}

#[derive(Clone, Copy)]
pub struct StandardPlonkConfig {
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Advice>,
    q_a: Column<Fixed>,
    q_b: Column<Fixed>,
    q_c: Column<Fixed>,
    q_ab: Column<Fixed>,
    constant: Column<Fixed>,
    #[allow(dead_code)]
    instance: Column<Instance>,
}

impl StandardPlonkConfig {
    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self {
        let [a, b, c] = [(); 3].map(|_| meta.advice_column());
        let [q_a, q_b, q_c, q_ab, constant] = [(); 5].map(|_| meta.fixed_column());
        let instance = meta.instance_column();

        [a, b, c].map(|column| meta.enable_equality(column));

        meta.create_gate(
            "q_a·a + q_b·b + q_c·c + q_ab·a·b + constant + instance = 0",
            |meta| {
                let [a, b, c] = [a, b, c].map(|column| meta.query_advice(column, Rotation::cur()));
                let [q_a, q_b, q_c, q_ab, constant] = [q_a, q_b, q_c, q_ab, constant]
                    .map(|column| meta.query_fixed(column, Rotation::cur()));
                let instance = meta.query_instance(instance, Rotation::cur());
                Some(
                    q_a * a.clone()
                        + q_b * b.clone()
                        + q_c * c
                        + q_ab * a * b
                        + constant
                        + instance,
                )
            },
        );

        StandardPlonkConfig {
            a,
            b,
            c,
            q_a,
            q_b,
            q_c,
            q_ab,
            constant,
            instance,
        }
    }
}

#[derive(Clone, Default)]
pub struct StandardPlonk(Fr);

impl StandardPlonk {
    pub fn rand<R: RngCore>(mut rng: R) -> Self {
        Self(Fr::from(rng.next_u32() as u64))
    }

    pub fn num_instance() -> Vec<usize> {
        vec![1]
    }

    pub fn instances(&self) -> Vec<Vec<Fr>> {
        vec![vec![self.0]]
    }
}

impl Circuit<Fr> for StandardPlonk {
    type Config = StandardPlonkConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        meta.set_minimum_degree(4);
        StandardPlonkConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "",
            |mut region| {
                region.assign_advice(|| "", config.a, 0, || Value::known(self.0))?;
                region.assign_fixed(|| "", config.q_a, 0, || Value::known(-Fr::one()))?;

                region.assign_advice(|| "", config.a, 1, || Value::known(-Fr::from(5)))?;
                for (idx, column) in (1..).zip([
                    config.q_a,
                    config.q_b,
                    config.q_c,
                    config.q_ab,
                    config.constant,
                ]) {
                    region.assign_fixed(|| "", column, 1, || Value::known(Fr::from(idx)))?;
                }

                let a = region.assign_advice(|| "", config.a, 2, || Value::known(Fr::one()))?;
                a.copy_advice(|| "", &mut region, config.b, 3)?;
                a.copy_advice(|| "", &mut region, config.c, 4)?;

                Ok(())
            },
        )
    }
}
