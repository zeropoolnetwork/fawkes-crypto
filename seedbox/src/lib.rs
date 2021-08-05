#![cfg_attr(not(feature = "std"), no_std)]

use blake2_rfc::blake2s::Blake2s;
use byteorder::{LittleEndian, ByteOrder};

pub const PERSONALIZATION: &[u8; 8] = b"__fawkes";

pub struct SeedboxBlake2 {
    salt: Option<[u8; 32]>,
    n_iter: u64,
    n_limb: usize,
    buff: [u8; 32],
}

impl SeedboxBlake2 {
    fn update(&mut self) {
        self.n_limb = 0;
        let mut h = Blake2s::with_params(32, &[], &[], PERSONALIZATION);
        let mut n_iter_bin = [0u8; 8];

        LittleEndian::write_u64(&mut n_iter_bin, self.n_iter);

        self.n_iter += 1;
        h.update(n_iter_bin.as_ref());
        if self.salt.is_some() {
            h.update(self.salt.unwrap().as_ref());
        }
        self.buff.as_mut().clone_from_slice(h.finalize().as_ref())
    }

    fn next_byte(&mut self) -> u8 {
        if self.n_limb == 32 {
            self.update();
        }

        let res = self.buff[self.n_limb];
        self.n_limb += 1;
        res
    }

}

pub trait SeedBox {
    fn fill_bytes(&mut self, dest: &mut [u8]);
    fn fill_limbs(&mut self, dest: &mut [u64]);
    fn new() -> Self;
    fn new_with_salt(salt: &[u8]) -> Self;
}

impl SeedBox for SeedboxBlake2 {
    fn new() -> Self {
        let mut res = SeedboxBlake2 {
            salt: None,
            n_iter: 0,
            n_limb: 8,
            buff: [0; 32],
        };
        res.update();
        res
    }

    fn new_with_salt(salt: &[u8]) -> Self {
        let mut h = Blake2s::new(32);
        let mut buff = [0u8; 32];
        h.update(salt);
        buff[..].clone_from_slice(h.finalize().as_ref());

        let mut res= SeedboxBlake2 {
            salt: Some(buff),
            n_iter: 0,
            n_limb: 8,
            buff: [0; 32],
        };
        res.update();
        res
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.iter_mut().for_each(|f| *f = self.next_byte());
    }

    fn fill_limbs(&mut self, dest: &mut [u64]) {
        let mut b = [0u8;8];

        dest.iter_mut().for_each(|f| {
            self.fill_bytes(&mut b);
            *f = LittleEndian::read_u64(&b);
        });
    }
}

pub trait SeedBoxGen<Out>: SeedBox {
    fn gen(&mut self) -> Out;
}

pub trait FromSeed<S:SeedBoxGen<Self>> : Sized {
    fn from_seed(seed: &[u8]) -> Self;
}

impl<Out, S> FromSeed<S> for Out where S:SeedBoxGen<Out> {
    fn from_seed(seed: &[u8]) -> Self {
        let mut sb = S::new_with_salt(seed);
        sb.gen()
    }
}
