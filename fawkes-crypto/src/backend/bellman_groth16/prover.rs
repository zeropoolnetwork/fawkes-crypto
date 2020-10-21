#[cfg(feature = "rand_support")]
use super::osrng::OsRng;
use super::*;
use bellman::{ConstraintSystem, SynthesisError};
pub struct Proof<E: Engine> {
    a: G1Point<E>,
    b: G2Point<E>,
    c: G1Point<E>,
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

pub fn convert_lc<E: Engine>(
    lc: &[(Num<E::Fr>, usize)],
    varmap: &[bellman::Variable],
) -> bellman::LinearCombination<E::BE> {
    let mut res = Vec::with_capacity(lc.len());

    for e in lc.iter() {
        let k = num_to_bellman_fp(e.0);
        let v = varmap[e.1];
        res.push((v, k));
    }
    bellman::LinearCombination::new(res)
}

impl<E: Engine> bellman::Circuit<E::BE> for BellmanCS<E> {
    fn synthesize<BCS: ConstraintSystem<E::BE>>(
        self,
        bellman_cs: &mut BCS,
    ) -> Result<(), SynthesisError> {
        let BellmanCS(rcs) = self;
        let cs = rcs.borrow();
        let mut public_indexes = cs.public.clone();
        public_indexes.sort();
        let mut public_indexes = public_indexes.into_iter();
        let vars_length = cs.values.len();
        let mut variables = Vec::with_capacity(vars_length);

        //build constant signal
        public_indexes.next().unwrap();
        variables.push(BCS::one());

        let mut i = 1;
        loop {
            let t = public_indexes.next();
            for j in i..t.unwrap_or(vars_length) {
                let v = bellman_cs
                    .alloc(
                        || format!("var_{}", j),
                        || {
                            cs.values[j]
                                .map(|v| num_to_bellman_fp(v))
                                .ok_or(SynthesisError::AssignmentMissing)
                        },
                    )
                    .unwrap();
                variables.push(v);
            }

            match t {
                Some(t) => {
                    let v = bellman_cs
                        .alloc_input(
                            || format!("var_{}", t),
                            || {
                                cs.values[t]
                                    .map(|v| num_to_bellman_fp(v))
                                    .ok_or(SynthesisError::AssignmentMissing)
                            },
                        )
                        .unwrap();
                    variables.push(v);
                    i = t + 1;
                }
                _ => break,
            }
        }

        for (i, g) in cs.gates.iter().enumerate().skip(1) {
            bellman_cs.enforce(
                || format!("constraint {}", i),
                |_| convert_lc::<E>(&g.0, &variables),
                |_| convert_lc::<E>(&g.1, &variables),
                |_| convert_lc::<E>(&g.2, &variables),
            );
        }
        Ok(())
    }
}

#[cfg(feature = "rand_support")]
pub fn prove<E: Engine, Pub: Signal<E::Fr>, Sec: Signal<E::Fr>, C: Fn(Pub, Sec)>(
    params: &Parameters<E>,
    input_pub: &Pub::Value,
    input_sec: &Sec::Value,
    circuit: C,
) -> (Vec<Num<E::Fr>>, Proof<E>) {
    let ref rcs = CS::rc_new(false);
    let signal_pub = Pub::alloc(rcs, Some(input_pub));
    signal_pub.inputize();
    let signal_sec = Sec::alloc(rcs, Some(input_sec));

    circuit(signal_pub, signal_sec);

    let bcs = BellmanCS::<E>(rcs.clone());

    let ref mut rng = OsRng::new();
    let proof =
        Proof::from_bellman(&bellman::groth16::create_random_proof(bcs, &params.0, rng).unwrap());
    let values = &rcs.borrow().values;
    let pub_indexes = &rcs.borrow().public;
    let inputs = pub_indexes
        .iter()
        .skip(1)
        .map(|&i| values[i].unwrap())
        .collect();
    (inputs, proof)
}
