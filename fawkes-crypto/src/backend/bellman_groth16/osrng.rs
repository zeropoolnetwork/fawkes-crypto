use bellman::rand::Rng;
use getrandom::getrandom;

#[derive(Default)]
pub struct OsRng;

impl Rng for OsRng {
    fn next_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        getrandom(&mut buf).unwrap();
        u32::from_be_bytes(buf)
    }
}
