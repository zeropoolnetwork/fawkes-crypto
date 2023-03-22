#[cfg(feature = "r1cs")]
pub mod tests {
    use fawkes_crypto::{
        circuit::{cs::{DebugCS, CS}, num::CNum},
        core::{signal::Signal},
        engines::bn256::Fr,
        rand::{thread_rng, Rng},
    };



    #[test]
    fn test_numeric_multiplication() {
        let ref mut cs = DebugCS::<Fr>::rc_new();
        let mut rng = thread_rng();

        let _a = rng.gen();
        let _b = rng.gen();
        let _c = _a * _b * _b;

        let a = CNum::alloc(cs, Some(&_a));
        let b = CNum::alloc(cs, Some(&_b));

        let mut n_constraints = cs.borrow().num_gates();
        let c = a * &b * b;
        n_constraints = cs.borrow().num_gates() - n_constraints;

        println!("a * b^2 == c constraints = {}", n_constraints);
        assert!(c.get_value().unwrap() == _c);
    }
}
