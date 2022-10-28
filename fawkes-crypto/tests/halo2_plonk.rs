use fawkes_crypto::backend::halo2_plonk::prover::prove;
use halo2_curves::bn256::Bn256;
use halo2_proofs::{
    arithmetic::Field,
    circuit::{Layouter, SimpleFloorPlanner},
    plonk::{Circuit, ConstraintSystem, Error},
    poly::{commitment::ParamsProver, kzg::commitment::ParamsKZG},
    transcript::{Blake2bWrite, Challenge255},
};

#[test]
fn test_create_proof() {
    #[derive(Clone, Copy)]
    struct MyCircuit;

    impl<F: Field> Circuit<F> for MyCircuit {
        type Config = ();

        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            *self
        }

        fn configure(_meta: &mut ConstraintSystem<F>) -> Self::Config {}

        fn synthesize(
            &self,
            _config: Self::Config,
            _layouter: impl Layouter<F>,
        ) -> Result<(), Error> {
            Ok(())
        }
    }

    const K: u32 = 4;

    let params = ParamsKZG::<Bn256>::new(K);

    // Create proof with correct number of instances
    prove::<_, _, Blake2bWrite<_, _, Challenge255<_>>>(&params, MyCircuit, vec![]);
}
