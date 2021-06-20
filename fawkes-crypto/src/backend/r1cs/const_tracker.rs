use std::io::{self, Read, Write};

use bit_vec::BitVec;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

const MAGIC: &[u8; 4] = b"ZPCT";

pub struct ConstTrackerFile(pub BitVec);

impl ConstTrackerFile {
    pub fn read<R: Read>(mut r: R) -> io::Result<Self> {
        let mut magic = [0; MAGIC.len()];
        r.read_exact(&mut magic)?;

        if magic != *MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic sequence",
            ));
        }

        let len = r.read_u64::<LittleEndian>()?;
        let mut bytes = vec![0; div_ceil(len as usize, 8)];
        r.read_exact(&mut bytes)?;

        Ok(ConstTrackerFile(BitVec::from_bytes(&bytes)))
    }

    pub fn write<W: Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(MAGIC)?;
        w.write_u64::<LittleEndian>(self.0.len() as u64)?; // Number of bits

        let bytes = self.0.to_bytes();
        w.write_all(&bytes)?;

        Ok(())
    }
}

#[inline]
fn div_ceil(a: usize, b: usize) -> usize {
    let (q, r) = (a / b, a % b);
    if r == 0 {
        q
    } else {
        q + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFERENCE: &[u8] = &[
        b'Z', b'P', b'C', b'T', // magic
        0x0F,0,0,0,0,0,0,0, // length - 15 bits
        0b10100000, 0b00010010, // bits
    ];

    #[test]
    fn test_const_tracker_file_read() {
        let ct = ConstTrackerFile::read(REFERENCE).unwrap();

        let bytes = ct.0.to_bytes();
        assert_eq!(&bytes, &[0b10100000, 0b00010010]);
    }

    #[test]
    fn test_const_tracker_file_write() {
        let mut ct = ConstTrackerFile(BitVec::from_bytes(&[0b10100000, 0b00010010]));
        ct.0.pop();

        let mut buf = Vec::new();
        ct.write(&mut buf);

        assert_eq!(&buf, REFERENCE);
    }
}