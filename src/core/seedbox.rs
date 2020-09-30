use blake2_rfc::blake2s::Blake2s;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;

use crate::constants::PERSONALIZATION;

pub struct SeedboxBlake2 {
    salt: Option<[u8; 32]>,
    n_iter: u64,
    n_limb: usize,
    buff: [u8; 32],
}

impl SeedboxBlake2 {
    pub fn new() -> Self {
        SeedboxBlake2 {
            salt: None,
            n_iter: 0,
            n_limb: 8,
            buff: [0; 32],
        }
    }

    pub fn new_with_salt(salt: &[u8]) -> Self {
        let mut h = Blake2s::new(32);
        let mut buff = [0u8; 32];
        h.update(salt);
        buff[..].clone_from_slice(h.finalize().as_ref());

        SeedboxBlake2 {
            salt: Some(buff),
            n_iter: 0,
            n_limb: 8,
            buff: [0; 32],
        }
    }
}

impl Rng for SeedboxBlake2 {
    fn next_u32(&mut self) -> u32 {
        if self.n_limb == 8 {
            self.n_limb = 0;
            let mut h = Blake2s::with_params(32, &[], &[], PERSONALIZATION);
            let mut n_iter_bin = [0u8; 8];
            n_iter_bin
                .as_mut()
                .write_u64::<LittleEndian>(self.n_iter)
                .unwrap();
            self.n_iter += 1;
            h.update(n_iter_bin.as_ref());
            if self.salt.is_some() {
                h.update(self.salt.unwrap().as_ref());
            }
            self.buff.as_mut().clone_from_slice(h.finalize().as_ref())
        }

        let res = (&self.buff[self.n_limb * 4..(self.n_limb + 1) * 4])
            .read_u32::<LittleEndian>()
            .unwrap();
        self.n_limb += 1;
        res
    }
}
