use bit_vec::BitVec;
pub use r1cs_file::*;

use crate::circuit::cs::Gate;
use crate::circuit::lc::Index;
use crate::ff_uint::{Num, PrimeField, Uint};
use crate::backend::r1cs::ConstTrackerFile;

pub fn get_r1cs_file<Fr: PrimeField, const FS: usize>(
    gates: &Vec<Gate<Fr>>,
    consts: &BitVec,
) -> (R1csFile<FS>, ConstTrackerFile) {
    use r1cs_file::*;

    let mut n_pub_in = 0;
    let mut n_prvt_in = 0;

    let constraints = gates
        .iter()
        .map(|gate| {
            let mut map_comb = |(c, i): &(Num<Fr>, Index)| {
                let i = match *i {
                    Index::Input(i) => {
                        n_pub_in += 1;
                        i
                    }
                    Index::Aux(i) => {
                        n_prvt_in += 1;
                        i
                    }
                };

                let mut c_bytes = [0; FS];
                c.0.to_uint().put_little_endian(&mut c_bytes);

                (FieldElement::from(c_bytes), i as u32)
            };

            let a = gate.0.iter().map(&mut map_comb).collect();
            let b = gate.1.iter().map(&mut map_comb).collect();
            let c = gate.2.iter().map(&mut map_comb).collect();

            Constraint(a, b, c)
        })
        .collect();

    let mut prime_bytes = [0; FS];
    Fr::MODULUS.put_little_endian(&mut prime_bytes);

    let r1cs_file = R1csFile {
        header: Header {
            prime: FieldElement::from(prime_bytes),
            n_wires: n_pub_in + n_prvt_in,
            n_pub_out: 0,
            n_pub_in,
            n_prvt_in,
            n_labels: 0,
            n_constraints: gates.len() as u32,
        },
        constraints: Constraints(constraints),
        map: WireMap(Vec::new()),
    };

    let consts = ConstTrackerFile(consts.clone());

    (r1cs_file, consts)
}

#[cfg(test)]
mod tests {
    use crate::{
        backend::bellman_groth16::{
            engines::Bn256,
            setup::setup
        },
        circuit::cs::CS,
        circuit::num::CNum,
        circuit::poseidon::{c_poseidon_merkle_proof_root, CMerkleProof},
        core::signal::Signal,
        native::poseidon::{PoseidonParams},
    };
    use crate::backend::r1cs::get_r1cs_file;
    use crate::backend::bellman_groth16::engines::Engine;

    #[test]
    fn test_parameters_get_r1cs_file() {
        fn circuit<C:CS>(public: CNum<C>, secret: (CNum<C>, CMerkleProof<C, 32>)) {
            let poseidon_params = PoseidonParams::<C::Fr>::new(3, 8, 53);
            let res = c_poseidon_merkle_proof_root(&secret.0, &secret.1, &poseidon_params);
            res.assert_eq(&public);
        }

        let params = setup::<Bn256, _, _, _>(circuit);
        let file = get_r1cs_file::<<Bn256 as Engine>::Fr, 32>(&params.1, &params.2);

        assert!(true)
    }
}