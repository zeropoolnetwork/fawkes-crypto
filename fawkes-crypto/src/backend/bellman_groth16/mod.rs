//use bellman::groth16::{Proof, Parameters};

trait WithBellmanEngine {
    type E: bellman::pairing::Engine;
}

pub mod osrng;
