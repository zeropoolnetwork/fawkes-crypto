
use fawkes_crypto::{
    circuit::{bitify::c_into_bits_le_strict, cs::DebugCS, ecc::*, num::CNum, cs::CS},
    native::ecc::*,
    ff_uint::Num,
    core::signal::Signal,
    engines::bn256::{Fr, JubJubBN256},
    rand::{thread_rng, Rng},
};

#[test]
fn test_scalar_point_picker() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let t = rng.gen();

    let ref mut cs = DebugCS::rc_new();
    let signal_t = CNum::alloc(cs, Some(&t));

    let signal_p = CEdwardsPoint::from_scalar(&signal_t, &jubjub_params);
    let p = EdwardsPoint::from_scalar(t, &jubjub_params);

    signal_p.assert_const(&p);
}

#[test]
fn test_circuit_subgroup_decompress() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p =
        EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params).mul(Num::from(8), &jubjub_params);

    let ref mut cs = DebugCS::rc_new();
    let signal_x = CNum::alloc(cs, Some(&p.x));

    let mut n_constraints = cs.borrow().num_gates();
    let res = CEdwardsPoint::subgroup_decompress(&signal_x, &jubjub_params);
    n_constraints = cs.borrow().num_gates() - n_constraints;

    res.y.assert_const(&p.y);

    println!("subgroup_decompress constraints = {}", n_constraints);

    assert!(res.y.get_value().unwrap() == p.y);
}

#[test]
fn test_circuit_edwards_add() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p1 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
    let p2 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);

    let p3 = p1.add(&p2, &jubjub_params);

    let ref mut cs = DebugCS::rc_new();
    let signal_p1 = CEdwardsPoint::alloc(cs, Some(&p1));
    let signal_p2 = CEdwardsPoint::alloc(cs, Some(&p2));

    let signal_p3 = signal_p1.add(&signal_p2, &jubjub_params);

    signal_p3.assert_const(&p3);
}

#[test]
fn test_circuit_edwards_double() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);

    let p3 = p.double();

    let ref mut cs = DebugCS::rc_new();
    let signal_p = CEdwardsPoint::alloc(cs, Some(&p));

    let signal_p3 = signal_p.double(&jubjub_params);

    signal_p3.assert_const(&p3);
}

#[test]
fn test_circuit_edwards_into_montgomery() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();
    let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
    let mp = p.into_montgomery().unwrap();
    let ref mut cs = DebugCS::rc_new();
    let signal_p = CEdwardsPoint::alloc(cs, Some(&p));
    let signal_mp = signal_p.into_montgomery();
    signal_mp.assert_const(&mp);
}

#[test]
fn test_circuit_montgomery_into_edwards() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
    let mp = p.into_montgomery().unwrap();
    let ref mut cs = DebugCS::rc_new();
    let signal_mp = CMontgomeryPoint::alloc(cs, Some(&mp));
    let signal_p = signal_mp.into_edwards();

    signal_p.assert_const(&p);
}

#[test]
fn test_circuit_montgomery_add() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p1 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
    let p2 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);

    let p3 = p1.add(&p2, &jubjub_params);

    let ref mut cs = DebugCS::rc_new();
    let signal_p1 = CEdwardsPoint::alloc(cs, Some(&p1));
    let signal_p2 = CEdwardsPoint::alloc(cs, Some(&p2));

    let signal_mp1 = signal_p1.into_montgomery();
    let signal_mp2 = signal_p2.into_montgomery();

    let signal_mp3 = signal_mp1.add(&signal_mp2, &jubjub_params);
    let signal_p3 = signal_mp3.into_edwards();

    signal_p3.assert_const(&p3);
}

#[test]
fn test_circuit_montgomery_double() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
    let p3 = p.double();

    let ref mut cs = DebugCS::rc_new();
    let signal_p = CEdwardsPoint::alloc(cs, Some(&p));
    let signal_mp = signal_p.into_montgomery();
    let signal_mp3 = signal_mp.double(&jubjub_params);
    let signal_p3 = signal_mp3.into_edwards();

    signal_p3.assert_const(&p3);
}

#[test]
fn test_circuit_edwards_mul() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params)
        .mul(Num::from(8), &jubjub_params);
    let n: Num<Fr> = rng.gen();

    let p3 = p.mul(n.to_other_reduced(), &jubjub_params);

    let ref mut cs = DebugCS::rc_new();
    let signal_p = CEdwardsPoint::alloc(cs, Some(&p));
    let signal_n = CNum::alloc(cs, Some(&n));

    let signal_n_bits = c_into_bits_le_strict(&signal_n);

    let mut n_constraints = cs.borrow().num_gates();
    let signal_p3 = signal_p.mul(&signal_n_bits, &jubjub_params);
    n_constraints = cs.borrow().num_gates() - n_constraints;

    signal_p3.assert_const(&p3);
    println!("edwards_mul constraints = {}", n_constraints);
}

#[test]
fn test_circuit_edwards_mul_const() {
    let mut rng = thread_rng();
    let jubjub_params = JubJubBN256::new();

    let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params)
        .mul(Num::from(8), &jubjub_params);
    let n: Num<Fr> = rng.gen();

    let p3 = p.mul(n.to_other_reduced(), &jubjub_params);

    let ref mut cs = DebugCS::rc_new();
    let signal_p = CEdwardsPoint::from_const(cs, &p);
    let signal_n = CNum::alloc(cs, Some(&n));

    let signal_n_bits = c_into_bits_le_strict(&signal_n);

    let mut n_constraints = cs.borrow().num_gates();
    let signal_p3 = signal_p.mul(&signal_n_bits, &jubjub_params);
    n_constraints = cs.borrow().num_gates() - n_constraints;

    signal_p3.assert_const(&p3);

    println!("edwards_mul_const constraints = {}", n_constraints);
}

