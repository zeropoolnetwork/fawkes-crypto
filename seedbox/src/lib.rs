#![cfg_attr(not(feature = "std"), no_std)]

use sha3::{Digest, Keccak256};
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

pub const PERSONALIZATION: &'static [u8; 8] = b"__fawkes";

fn keccak256(data:&[u8])->[u8;32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let mut res = [0u8;32];
    res.iter_mut().zip(hasher.finalize().into_iter()).for_each(|(l,r)| *l=r);
    res
}


pub struct SeedboxChaCha20(ChaCha20Rng);


pub trait SeedBox {
    fn fill_bytes(&mut self, dest: &mut [u8]);
    fn fill_limbs(&mut self, dest: &mut [u64]);
    fn new_with_salt(salt: &[u8]) -> Self;
}

impl SeedBox for SeedboxChaCha20 {
    fn new_with_salt(salt: &[u8]) -> Self {
        SeedboxChaCha20(<ChaCha20Rng as SeedableRng>::from_seed(keccak256(salt)))
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    fn fill_limbs(&mut self, dest: &mut [u64]) {
        dest.iter_mut().for_each(|f| *f = self.0.next_u64());
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
