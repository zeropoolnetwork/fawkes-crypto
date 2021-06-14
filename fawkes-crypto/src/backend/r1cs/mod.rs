use std::io::{self, Read, Write};

use bit_vec::BitVec;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
pub use r1cs_file::*;
pub use wtns_file::WtnsFile;

use crate::circuit::cs::Gate;
use crate::circuit::lc::Index;
use crate::core::signal::Signal;
use crate::ff_uint::{Num, PrimeField, Uint};

// TODO: Separate into multiple modules?

const C_MAGIC: &[u8; 8] = b"ZPCCONST";

pub struct ConstTracker(pub BitVec);

impl ConstTracker {
    pub fn read<R: Read>(mut r: R) -> io::Result<Self> {
        let mut magic = [u8; 8];
        r.read_exact(&mut magic)?;

        if magic != *C_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic sequence",
            ));
        }

        let len = r.read_u64::<LittleEndian>()?;
        let mut bytes = vec![0; len as usize];
        r.read_exact(&mut bytes);

        Ok(ConstTracker(BitVec::from_bytes(&bytes)))
    }

    pub fn write<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(MAGIC)?;
        w.write_u64::<LittleEndian>(bytes.len() as u64)?;

        let bytes = self.0.to_bytes();
        w.write_all(&bytes)?;

        Ok(())
    }
}

pub fn get_r1cs_file<Fr: PrimeField, const FS: usize>(
    gates: &Vec<Gate<Fr>>,
    consts: &BitVec,
) -> (R1csFile<FS>, ConstTracker) {
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

    let consts = ConstTracker(consts.clone());

    (r1cs_file, consts)
}

#[cfg(feature = "wtns-file")]
pub fn get_witness<
    'a,
    Fr: PrimeField,
    Pub: Signal<WitnessCS<'a, Fr>>,
    Sec: Signal<WitnessCS<'a, Fr>>,
    const FS: usize,
>(
    gates: &Vec<Gate<Fr>>,
    consts: &BitVec,
    input_pub: &Pub::Value,
    input_sec: &Pub::Value,
) -> WtnsFile {
    let cs = WitnessCS::rc_new(gates, consts);

    let signal_pub = Pub::alloc(cs, Some(input_pub));
    signal_pub.inputize();
    let signal_sec = Sec::alloc(cs, Some(input_sec));
}
