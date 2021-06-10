use crate::circuit::lc::Index;
use crate::circuit::cs::Gate;
use crate::ff_uint::{Num, PrimeField, Uint};

pub use r1cs_file::*;

#[cfg(feature = "r1cs-file")]
pub fn get_r1cs_file<Fr: PrimeField, const FS: usize>(gates: &Vec<Gate<Fr>>) -> R1csFile<FS> {
    use r1cs_file::*;

    let mut n_pub_in = 0;
    let mut n_prvt_in = 0;

    let constraints = gates.iter().map(|gate| {
        let mut map_comb = |(c, i): &(Num<Fr>, Index)| {
            let i = match *i {
                Index::Input(i) => {
                    n_pub_in += 1;
                    i
                },
                Index::Aux(i) => {
                    n_prvt_in += 1;
                    i
                },
            };


            let mut c_bytes = [0; FS];
            c.0.to_uint().put_little_endian(&mut c_bytes);

            (FieldElement::from(c_bytes), i as u32)
        };

        let a = gate.0.iter().map(&mut map_comb).collect();
        let b = gate.1.iter().map(&mut map_comb).collect();
        let c = gate.2.iter().map(&mut map_comb).collect();

        Constraint(a, b, c)
    }).collect();

    let mut prime_bytes = [0; FS];
    Fr::MODULUS.put_little_endian(&mut prime_bytes);

    R1csFile {
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
    }
}