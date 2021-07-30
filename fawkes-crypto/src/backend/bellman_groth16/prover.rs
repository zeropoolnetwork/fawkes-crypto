#[cfg(feature = "rand_support")]
use super::osrng::OsRng;
use super::*;
use super::group::{G1Point, G2Point};

#[cfg(feature = "serde_support")]
use serde::{Serialize, Deserialize};
#[cfg(feature = "borsh_support")]
use borsh::{BorshSerialize, BorshDeserialize};

#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "", deserialize = "")))]
pub struct Proof<E: Engine> {
    pub a: G1Point<E>,
    pub b: G2Point<E>,
    pub c: G1Point<E>,
}

impl<E: Engine> Proof<E> {
    pub fn to_bellman(&self) -> bellman::groth16::Proof<E::BE> {
        bellman::groth16::Proof {
            a: self.a.to_bellman(),
            b: self.b.to_bellman(),
            c: self.c.to_bellman(),
        }
    }

    pub fn from_bellman(proof: &bellman::groth16::Proof<E::BE>) -> Self {
        Self {
            a: G1Point::from_bellman(&proof.a),
            b: G2Point::from_bellman(&proof.b),
            c: G1Point::from_bellman(&proof.c),
        }
    }
}


#[cfg(feature = "borsh_support")]
impl<E: Engine> BorshSerialize for Proof<E> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.a, writer)?;
        BorshSerialize::serialize(&self.b, writer)?;
        BorshSerialize::serialize(&self.c, writer)
    }
}

#[cfg(feature = "borsh_support")]
impl<E: Engine> BorshDeserialize for Proof<E> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let a = BorshDeserialize::deserialize(buf)?;
        let b = BorshDeserialize::deserialize(buf)?;
        let c = BorshDeserialize::deserialize(buf)?;

        Ok(Self {
            a,
            b,
            c,
        })
    }
}

#[cfg(feature = "rand_support")]
pub fn prove<'a, E: Engine, Pub: Signal<WitnessCS<'a, E::Fr>>, Sec: Signal<WitnessCS<'a, E::Fr>>, C: Fn(Pub, Sec)>(
    params: &'a Parameters<E>,
    input_pub: &Pub::Value,
    input_sec: &Sec::Value,
    circuit: C,
) -> (Vec<Num<E::Fr>>, Proof<E>) {
    let ref rcs = params.get_witness_rcs();
    let signal_pub = Pub::alloc(rcs, Some(input_pub));
    signal_pub.inputize();
    let signal_sec = Sec::alloc(rcs, Some(input_sec));

    circuit(signal_pub, signal_sec);

    let bcs = BellmanCS::<E, WitnessCS<E::Fr>>::new(rcs.clone());

    let ref mut rng = OsRng::new();
    let proof =
        Proof::from_bellman(&bellman::groth16::create_random_proof(bcs, &params.0, rng).unwrap());

    let cs = rcs.borrow();
    assert!(cs.const_tracker_index==cs.const_tracker.len(), "not all cached data used");
    let mut inputs = Vec::with_capacity(cs.num_input());
    for i in 1..cs.num_input() as u32{
        inputs.push(cs.get_value(Index::Input(i)).unwrap())
    }
    
    (inputs, proof)
}
