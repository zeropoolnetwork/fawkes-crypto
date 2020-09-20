// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::convert::TryInto;
use core::str::FromStr;
use core::u64::MAX;
use ff_uint::{construct_uint, overflowing, Uint, construct_primefield_params, Field, SqrtField, LegendreSymbol};



construct_uint! {
		pub struct U256(4);
}

construct_uint! {
		pub struct U512(8);
}



construct_primefield_params! {
    pub struct Fs(super::U256);

    impl PrimeFieldParams for Fs {
        type Inner = super::U256;
        const MODULUS: &'static str = "6554484396890773809930967563523245729705921265872317281365359162392183254199";
        const GENERATOR: &'static str = "7";
   }
}

#[test]
fn ff_add() {
	let el_1 = Fs::from("5323078204692426412842508782764263339826862807415986655353573052856443119100");
	let el_2 = Fs::from("4987071179487603678244356207423490305353537992883325508024429714659828355784");

	assert_eq!(el_1 + el_2, Fs::from("3755664987289256281155897426664507915474479534426994882012643605124088220685"));
}

#[test]
fn ff_sub() {
	let el_1 = Fs::from("5522176803114537851033596417952985568305702500093007198964692595538719045489");
	let el_2 = Fs::from("4436391007479561855252505923592519356713124556528212035827942667719191650441");

	assert_eq!(el_1 - el_2, Fs::from("1085785795634975995781090494360466211592577943564795163136749927819527395048"));
}

#[test]
fn ff_mul() {
	let el_1 = Fs::from("6309289652141936190746119273485978351753073401847546942597907876037438057717");
	let el_2 = Fs::from("1835052206467827630361812248678048137284175571809216262414695473180494802642");

	assert_eq!(el_1 * el_2, Fs::from("4923613953693195297120254491542970414116901670530905390448508666798834089150"));
}

#[test]
fn ff_div() {
	let el_1 = Fs::from("5274802059733526156906427493357594382539576885437269793277325391175786253445");
	let el_2 = Fs::from("5024284913098887843516840674239024263531292519716163278998027572334671321838");

	assert_eq!(el_1 / el_2, Fs::from("1273923491188751922968527059783956164162684078496765223346152844261628009763"));
}

#[test]
fn ff_pow() {
	let el = Fs::from("3906975254792992609559966361868855986657674176119057801360690466200782403584");
	let exp = U256::from("5466750629119678727643417572265258306939894440271558996939980668336521407292");

	assert_eq!(
		el.pow(exp),
		Fs::from("1344500309604191514295302933545991495373814034913363937447159152611152415074"),
	);
}

#[test]
fn ff_legendre_zero() {
	let el = Fs::from("0");
	assert_eq!(el.legendre(), LegendreSymbol::Zero);
}

#[test]
fn ff_legendre_res() {
	let el = Fs::from("3190267433864704882419135144654036817987378091369085301042316138664348495392");
	assert_eq!(el.legendre(), LegendreSymbol::QuadraticResidue);
}

#[test]
fn ff_legendre_non_res() {
	let el = Fs::from("6041532138638958034213005325255886032699521298311442156907122277371035299984");
	assert_eq!(el.legendre(), LegendreSymbol::QuadraticNonResidue);
}

#[test]
fn ff_sqrt() {
	let el = Fs::from("6552443876041780908477089558487370394192470367232421400138897474988810492790");
	assert_eq!(el.sqrt(), Some(Fs::from("5846233863389012164445642602664269536052308164881483806310694309510474452608")));
}

#[test]
fn ff_sqrt_none() {
	let el = Fs::from("2536343238065325936731020634782488642997854861989312872507366216293166742491");
	assert_eq!(el.sqrt(), None);
}

#[test]
fn ff_neg_zero() {
	let el = Fs::from("0");
	assert_eq!(-el, Fs::from("0"));
}

#[test]
fn ff_neg_overflow() {
	let el = Fs::from("4333023617456302974597068220103947981834071240924067119638717307916415546782");
	assert_eq!(-el, Fs::from("2221460779434470835333899343419297747871850024948250161726641854475767707417"));
}

#[test]
fn hash_impl_is_the_same_as_for_a_slice() {
	use core::hash::{Hash, Hasher as _};
	use std::collections::hash_map::DefaultHasher;

	let uint_hash = {
		let mut h = DefaultHasher::new();
		let uint = U256::from(123u64);
		Hash::hash(&uint, &mut h);
		h.finish()
	};
	let slice_hash = {
		let mut h = DefaultHasher::new();
		Hash::hash(&[123u64, 0, 0, 0], &mut h);
		h.finish()
	};
	assert_eq!(uint_hash, slice_hash);
}



#[test]
fn uint256_checked_ops() {
	let z = U256::from(0);
	let a = U256::from(10);
	let b = !U256::from(1);

	assert_eq!(U256::from(10).checked_pow(U256::from(0)), Some(U256::from(1)));
	assert_eq!(U256::from(10).checked_pow(U256::from(1)), Some(U256::from(10)));
	assert_eq!(U256::from(10).checked_pow(U256::from(2)), Some(U256::from(100)));
	assert_eq!(U256::from(10).checked_pow(U256::from(3)), Some(U256::from(1000)));
	assert_eq!(U256::from(2).checked_pow(U256::from(0x100)), None);
	assert_eq!(U256::max_value().checked_pow(U256::from(2)), None);

	assert_eq!(a.checked_add(b), None);
	assert_eq!(a.checked_add(a), Some(20.into()));

	assert_eq!(a.checked_sub(b), None);
	assert_eq!(a.checked_sub(a), Some(0.into()));

	assert_eq!(a.checked_mul(b), None);
	assert_eq!(a.checked_mul(a), Some(100.into()));

	assert_eq!(a.checked_div(z), None);
	assert_eq!(a.checked_div(a), Some(1.into()));

	assert_eq!(a.checked_rem(z), None);
	assert_eq!(a.checked_rem(a), Some(0.into()));

	assert_eq!(a.checked_neg(), None);
	assert_eq!(z.checked_neg(), Some(z));
}

#[test]
fn uint256_from() {
	let e = U256([10, 0, 0, 0]);

	// test unsigned initialization
	let ua = U256::from(10u8);
	let ub = U256::from(10u16);
	let uc = U256::from(10u32);
	let ud = U256::from(10u64);
	assert_eq!(e, ua);
	assert_eq!(e, ub);
	assert_eq!(e, uc);
	assert_eq!(e, ud);

	// test initialization from bytes
	let va = U256::from_big_endian(&[10u8][..]);
	assert_eq!(e, va);

	// more tests for initialization from bytes
	assert_eq!(U256([0x1010, 0, 0, 0]), U256::from_big_endian(&[0x10u8, 0x10][..]));
	assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_big_endian(&[0x12u8, 0xf0][..]));
	assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_big_endian(&[0, 0x12u8, 0xf0][..]));
	assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_big_endian(&[0, 0, 0, 0, 0, 0, 0, 0x12u8, 0xf0][..]));
	assert_eq!(U256([0x12f0, 1, 0, 0]), U256::from_big_endian(&[1, 0, 0, 0, 0, 0, 0, 0x12u8, 0xf0][..]));
	assert_eq!(
		U256([0x12f0, 1, 0x0910203040506077, 0x8090a0b0c0d0e0f0]),
		U256::from_big_endian(
			&[
				0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77, 0, 0,
				0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0x12u8, 0xf0
			][..]
		)
	);
	assert_eq!(
		U256([0x00192437100019fa, 0x243710, 0, 0]),
		U256::from_big_endian(&[0x24u8, 0x37, 0x10, 0, 0x19, 0x24, 0x37, 0x10, 0, 0x19, 0xfa][..])
	);

	// test initializtion from string
	let sa = U256::from_str("10").unwrap();
	assert_eq!(e, sa);
	assert_eq!(U256([0x1010, 0, 0, 0]), U256::from_str("4112").unwrap());
	assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_str("04848").unwrap());
	assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_str("0004848").unwrap());
	assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_str("00004848").unwrap());
	assert_eq!(U256([0x12f0, 1, 0, 0]), U256::from_str("18446744073709556464").unwrap());
	assert_eq!(
		U256([0x12f0, 1, 0x0910203040506077, 0x8090a0b0c0d0e0f0]),
		U256::from_str("58151579640658172662156317730167191628621057582570617906134234538764069245680").unwrap()
	);
}

#[test]
fn uint256_try_into_primitives() {
	macro_rules! try_into_uint_primitive_ok {
		($primitive: ty) => {
			assert_eq!(U256::from(10).try_into() as Result<$primitive, _>, Ok(<$primitive>::from(10u8)));
		};
	}
	try_into_uint_primitive_ok!(u8);
	try_into_uint_primitive_ok!(u16);
	try_into_uint_primitive_ok!(u32);
	try_into_uint_primitive_ok!(usize);
	try_into_uint_primitive_ok!(u64);
	try_into_uint_primitive_ok!(u128);

	macro_rules! try_into_iint_primitive_ok {
		($primitive: ty) => {
			assert_eq!(U256::from(10).try_into() as Result<$primitive, _>, Ok(<$primitive>::from(10i8)));
		};
	}
	try_into_iint_primitive_ok!(i8);
	try_into_iint_primitive_ok!(i16);
	try_into_iint_primitive_ok!(i32);
	try_into_iint_primitive_ok!(isize);
	try_into_iint_primitive_ok!(i64);
	try_into_iint_primitive_ok!(i128);

	macro_rules! try_into_primitive_err {
		($small: ty, $big: ty) => {
			assert_eq!(
				U256::from(<$small>::max_value() as $big + 1).try_into() as Result<$small, _>,
				Err(concat!("integer overflow when casting to ", stringify!($small)))
			);
		};
	}
	try_into_primitive_err!(u8, u16);
	try_into_primitive_err!(u16, u32);
	try_into_primitive_err!(u32, u64);
	try_into_primitive_err!(usize, u128);
	try_into_primitive_err!(u64, u128);
	assert_eq!(U256([0, 0, 1, 0]).try_into() as Result<u128, _>, Err("integer overflow when casting to u128"));
	try_into_primitive_err!(i8, i16);
	try_into_primitive_err!(i16, i32);
	try_into_primitive_err!(i32, i64);
	try_into_primitive_err!(isize, i128);
	try_into_primitive_err!(i64, i128);
	try_into_primitive_err!(i128, u128);
	assert_eq!(U256([0, 0, 1, 0]).try_into() as Result<i128, _>, Err("integer overflow when casting to i128"));
}

#[test]
fn uint256_to() {
	let dec = "58151579640658172662156317730167191628738331711503941148816114301233141912304";
	let uint = U256::from_str(dec).unwrap();
	let mut bytes = [0u8; 32];
	uint.put_big_endian(&mut bytes);
	let uint2 = U256::from_big_endian(&bytes[..]);
	assert_eq!(uint, uint2);
}

#[test]
fn uint256_bits_test() {
	assert_eq!(U256::from(0u64).bits(), 0);
	assert_eq!(U256::from(255u64).bits(), 8);
	assert_eq!(U256::from(256u64).bits(), 9);
	assert_eq!(U256::from(300u64).bits(), 9);
	assert_eq!(U256::from(60000u64).bits(), 16);
	assert_eq!(U256::from(70000u64).bits(), 17);

	//// Try to read the following lines out loud quickly
	let mut shl = U256::from(70000u64);
	shl = shl << 100;
	assert_eq!(shl.bits(), 117);
	shl = shl << 100;
	assert_eq!(shl.bits(), 217);
	shl = shl << 100;
	assert_eq!(shl.bits(), 0);

	//// Bit set check
	//// 01010
	assert!(!U256::from(10u8).bit(0));
	assert!(U256::from(10u8).bit(1));
	assert!(!U256::from(10u8).bit(2));
	assert!(U256::from(10u8).bit(3));
	assert!(!U256::from(10u8).bit(4));


}

#[test]
fn uint256_comp_test() {
	let small = U256([10u64, 0, 0, 0]);
	let big = U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
	let bigger = U256([0x9C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
	let biggest = U256([0x5C8C3EE70C644118u64, 0x0209E7378231E632, 0, 1]);

	assert!(small < big);
	assert!(big < bigger);
	assert!(bigger < biggest);
	assert!(bigger <= biggest);
	assert!(biggest <= biggest);
	assert!(bigger >= big);
	assert!(bigger >= small);
	assert!(small <= small);
	assert_eq!(small, small);
	assert_eq!(biggest, biggest);
	assert_ne!(big, biggest);
	assert_ne!(big, bigger);
}

#[test]
fn uint256_arithmetic_test() {
	let init = U256::from(0xDEADBEEFDEADBEEFu64);
	let copy = init;

	let add = init + copy;
	assert_eq!(add, U256([0xBD5B7DDFBD5B7DDEu64, 1, 0, 0]));
	// Bitshifts
	let shl = add << 88;
	assert_eq!(shl, U256([0u64, 0xDFBD5B7DDE000000, 0x1BD5B7D, 0]));
	let shr = shl >> 40;
	assert_eq!(shr, U256([0x7DDE000000000000u64, 0x0001BD5B7DDFBD5B, 0, 0]));
	// Increment
	let incr = shr + U256::from(1u64);
	assert_eq!(incr, U256([0x7DDE000000000001u64, 0x0001BD5B7DDFBD5B, 0, 0]));
	// Subtraction
	let sub = overflowing!(incr.overflowing_sub(init));
	assert_eq!(sub, U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));
	// Multiplication
	let mult = sub * 300u64;
	assert_eq!(mult, U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]));
	// Division
	assert_eq!(U256::from(105u8) / U256::from(5u8), U256::from(21u8));
	let div = mult / U256::from(300u16);
	assert_eq!(div, U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));

	let a = U256::from_str("115339776388732929035197660848497720713218148788040405586178452820382218977489").unwrap();
	let b = U256::from_str("452312848583266388373324160190187140051835877600158453279131187530910662446").unwrap();
	println!("{:x}", a);
	println!("{:x}", b);
	assert_eq!(!a, b);
	assert_eq!(a, !b);
}

#[test]
fn uint256_simple_mul() {
	let a = U256::from_str("18446744073709551616").unwrap();
	let b = U256::from_str("18446744073709551616").unwrap();

	let c = U256::from_str("340282366920938463463374607431768211456").unwrap();
	println!("Multiplying");
	let result = a.overflowing_mul(b);
	println!("Got result");
	assert_eq!(result, (c, false))
}

#[test]
fn uint256_extreme_bitshift_test() {
	//// Shifting a u64 by 64 bits gives an undefined value, so make sure that
	//// we're doing the Right Thing here
	let init = U256::from(0xDEADBEEFDEADBEEFu64);

	assert_eq!(init << 64, U256([0, 0xDEADBEEFDEADBEEF, 0, 0]));
	let add = (init << 64) + init;
	assert_eq!(add, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
	assert_eq!(add >> 0, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
	assert_eq!(add << 0, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
	assert_eq!(add >> 64, U256([0xDEADBEEFDEADBEEF, 0, 0, 0]));
	assert_eq!(add << 64, U256([0, 0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0]));
}


#[test]
fn uint256_mul64() {
	assert_eq!(U256::from(0u64) * 2u64, U256::from(0u64));
	assert_eq!(U256::from(1u64) * 2u64, U256::from(2u64));
	assert_eq!(U256::from(10u64) * 2u64, U256::from(20u64));
	assert_eq!(U256::from(10u64) * 5u64, U256::from(50u64));
	assert_eq!(U256::from(1000u64) * 50u64, U256::from(50000u64));
}

#[test]
fn uint256_pow() {
	assert_eq!(U256::from(10).wrapping_pow(U256::from(0)), U256::from(1));
	assert_eq!(U256::from(10).wrapping_pow(U256::from(1)), U256::from(10));
	assert_eq!(U256::from(10).wrapping_pow(U256::from(2)), U256::from(100));
	assert_eq!(U256::from(10).wrapping_pow(U256::from(3)), U256::from(1000));
}

#[test]
#[should_panic]
fn uint256_pow_overflow_panic() {
	U256::from(2).checked_pow(U256::from(0x100)).unwrap();
}

#[test]
fn should_format_and_debug_correctly() {
	let test = |x: usize, hex: &'static str, display: &'static str| {
		assert_eq!(format!("{}", U256::from(x)), display);
		// TODO: proper impl for Debug so we get this to pass:  assert_eq!(format!("{:?}", U256::from(x)), format!("0x{}", hex));
		assert_eq!(format!("{:?}", U256::from(x)), display);
		assert_eq!(format!("{:x}", U256::from(x)), hex);
		assert_eq!(format!("{:#x}", U256::from(x)), format!("0x{}", hex));
	};

	test(0x1, "1", "1");
	test(0xf, "f", "15");
	test(0x10, "10", "16");
	test(0xff, "ff", "255");
	test(0x100, "100", "256");
	test(0xfff, "fff", "4095");
	test(0x1000, "1000", "4096");
}

#[test]
pub fn display_u256() {
	let expected = "115792089237316195423570985008687907853269984665640564039457584007913129639935";
	let value = U256::MAX;
	assert_eq!(format!("{}", value), expected);
	assert_eq!(format!("{:?}", value), expected);
}

#[test]
pub fn display_u512() {
	let expected = "13407807929942597099574024998205846127479365820592393377723561443721764030073546976801874298166903427690031858186486050853753882811946569946433649006084095";
	let value = U512::MAX;
	assert_eq!(format!("{}", value), expected);
	assert_eq!(format!("{:?}", value), expected);
}

#[test]
fn uint256_overflowing_pow() {
	assert_eq!(
		U256::from(2).overflowing_pow(U256::from(0xff)),
		(U256::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819968").unwrap(), false)
	);
	assert_eq!(U256::from(2).overflowing_pow(U256::from(0x100)), (U256::ZERO, true));
}

#[test]
fn uint256_mul1() {
	assert_eq!(U256::from(1u64) * U256::from(10u64), U256::from(10u64));
}

#[test]
fn uint256_mul2() {
	let a = U512::from_str("340282366920938463481821351505477763070").unwrap();
	let b = U512::from_str("340282366920938463463374607431768211455").unwrap();

	assert_eq!(a * b, U512::from_str("115792089237316195429848086744074588616084926988085415065151368886008149966850").unwrap());
}

#[test]
fn uint256_overflowing_mul() {
	assert_eq!(
		U256::from_str("340282366920938463463374607431768211456")
			.unwrap()
			.overflowing_mul(U256::from_str("340282366920938463463374607431768211456").unwrap()),
		(U256::ZERO, true)
	);
}

#[test]
fn uint512_mul() {
	assert_eq!(
		U512::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap()
		*
		U512::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap(),
		U512::from_str("3351951982485649274893506249551461531869841455148098344430890360930441007518270952111231258346302285937499276638768242728772830138947184902600499121881089").unwrap()
	);
}

#[test]
fn uint256_mul_overflow() {
	assert_eq!(
		U256::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap().overflowing_mul(
			U256::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap()
		),
		(U256::from_str("1").unwrap(), true)
	);
}

#[test]
#[should_panic]
#[allow(unused_must_use)]
fn uint256_mul_overflow_panic() {
	U256::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap()
		* U256::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap();
}

#[test]
fn uint256_sub_overflow() {
	assert_eq!(
		U256::from_str("0").unwrap().overflowing_sub(U256::from_str("1").unwrap()),
		(U256::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap(), true)
	);
}

#[test]
#[should_panic]
#[allow(unused_must_use)]
fn uint256_sub_overflow_panic() {
	U256::from_str("0").unwrap() - U256::from_str("1").unwrap();
}

#[test]
fn uint256_shl() {
	assert_eq!(
		U256::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap() << 4,
		U256::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639920").unwrap()
	);
}

#[test]
fn uint256_shl_words() {
	assert_eq!(
		U256::from_str("12554203470773361527671578846415332832204710888928069025791").unwrap() << 64,
		U256::from_str("115792089237316195423570985008687907853269984665640564039439137263839420088320").unwrap()
	);
	assert_eq!(
		U256::from_str("6277101735386680763835789423207666416102355444464034512895").unwrap() << 64,
		U256::from_str("115792089237316195423570985008687907853269984665640564039439137263839420088320").unwrap()
	);
}

#[test]
fn uint256_mul() {
	assert_eq!(
		U256::from_str("57896044618658097711785492504343953926634992332820282019728792003956564819967").unwrap()
			* U256::from_str("2").unwrap(),
		U256::from_str("115792089237316195423570985008687907853269984665640564039457584007913129639934").unwrap()
	);
}

#[test]
fn uint256_div() {
	assert_eq!(U256::from(10u64) / U256::from(1u64), U256::from(10u64));
	assert_eq!(U256::from(10u64) / U256::from(2u64), U256::from(5u64));
	assert_eq!(U256::from(10u64) / U256::from(3u64), U256::from(3u64));
}

#[test]
fn uint256_rem() {
	assert_eq!(U256::from(10u64) % U256::from(1u64), U256::from(0u64));
	assert_eq!(U256::from(10u64) % U256::from(3u64), U256::from(1u64));
}

#[test]
fn uint256_from_str() {
	assert_eq!(U256::from_str("10").unwrap(), U256::from(10u64));
	assert_eq!(U256::from_str("1024").unwrap(), U256::from(1024u64));
}

#[test]
fn display_uint() {
	let s = "12345678987654321023456789";
	assert_eq!(format!("{}", U256::from_str(s).unwrap()), s);
}

#[test]
fn display_uint_zero() {
	assert_eq!(format!("{}", U256::from(0)), "0");
}

#[test]
fn u512_multi_adds() {
	let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 0]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 0]));
	assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 0]));

	let (result, _) = U512([1, 0, 0, 0, 0, 0, 0, 1]).overflowing_add(U512([1, 0, 0, 0, 0, 0, 0, 1]));
	assert_eq!(result, U512([2, 0, 0, 0, 0, 0, 0, 2]));

	let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 1]));
	assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 2]));

	let (result, _) = U512([0, 0, 0, 0, 0, 0, 2, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 3, 1]));
	assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 5, 2]));

	let (result, _) = U512([1, 2, 3, 4, 5, 6, 7, 8]).overflowing_add(U512([9, 10, 11, 12, 13, 14, 15, 16]));
	assert_eq!(result, U512([10, 12, 14, 16, 18, 20, 22, 24]));

	let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 2, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 3, 1]));
	assert!(!overflow);

	let (_, overflow) =
		U512([MAX, MAX, MAX, MAX, MAX, MAX, MAX, MAX]).overflowing_add(U512([MAX, MAX, MAX, MAX, MAX, MAX, MAX, MAX]));
	assert!(overflow);

	let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 0, MAX]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, MAX]));
	assert!(overflow);

	let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 0, MAX]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 0]));
	assert!(!overflow);
}

#[test]
fn u256_multi_adds() {
	let (result, _) = U256([0, 0, 0, 0]).overflowing_add(U256([0, 0, 0, 0]));
	assert_eq!(result, U256([0, 0, 0, 0]));

	let (result, _) = U256([0, 0, 0, 1]).overflowing_add(U256([0, 0, 0, 1]));
	assert_eq!(result, U256([0, 0, 0, 2]));

	let (result, overflow) = U256([0, 0, 2, 1]).overflowing_add(U256([0, 0, 3, 1]));
	assert_eq!(result, U256([0, 0, 5, 2]));
	assert!(!overflow);

	let (_, overflow) = U256([MAX, MAX, MAX, MAX]).overflowing_add(U256([MAX, MAX, MAX, MAX]));
	assert!(overflow);

	let (_, overflow) = U256([0, 0, 0, MAX]).overflowing_add(U256([0, 0, 0, MAX]));
	assert!(overflow);
}

#[test]
fn u256_multi_subs() {
	let (result, _) = U256([0, 0, 0, 0]).overflowing_sub(U256([0, 0, 0, 0]));
	assert_eq!(result, U256([0, 0, 0, 0]));

	let (result, _) = U256([0, 0, 0, 1]).overflowing_sub(U256([0, 0, 0, 1]));
	assert_eq!(result, U256([0, 0, 0, 0]));

	let (_, overflow) = U256([0, 0, 2, 1]).overflowing_sub(U256([0, 0, 3, 1]));
	assert!(overflow);

	let (result, overflow) = U256([MAX, MAX, MAX, MAX]).overflowing_sub(U256([MAX / 2, MAX / 2, MAX / 2, MAX / 2]));

	assert!(!overflow);
	assert_eq!(U256([MAX / 2 + 1, MAX / 2 + 1, MAX / 2 + 1, MAX / 2 + 1]), result);

	let (result, overflow) = U256([0, 0, 0, 1]).overflowing_sub(U256([0, 0, 1, 0]));
	assert!(!overflow);
	assert_eq!(U256([0, 0, MAX, 0]), result);

	let (result, overflow) = U256([0, 0, 0, 1]).overflowing_sub(U256([1, 0, 0, 0]));
	assert!(!overflow);
	assert_eq!(U256([MAX, MAX, MAX, 0]), result);
}

#[test]
fn u512_multi_subs() {
	let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 0]).overflowing_sub(U512([0, 0, 0, 0, 0, 0, 0, 0]));
	assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 0]));

	let (result, _) = U512([10, 9, 8, 7, 6, 5, 4, 3]).overflowing_sub(U512([9, 8, 7, 6, 5, 4, 3, 2]));
	assert_eq!(result, U512([1, 1, 1, 1, 1, 1, 1, 1]));

	let (_, overflow) = U512([10, 9, 8, 7, 6, 5, 4, 3]).overflowing_sub(U512([9, 8, 7, 6, 5, 4, 3, 2]));
	assert!(!overflow);

	let (_, overflow) = U512([9, 8, 7, 6, 5, 4, 3, 2]).overflowing_sub(U512([10, 9, 8, 7, 6, 5, 4, 3]));
	assert!(overflow);
}

#[test]
fn u256_multi_carry_all() {
	let (result, _) = U256([MAX, 0, 0, 0]).overflowing_mul(U256([MAX, 0, 0, 0]));
	assert_eq!(U256([1, MAX - 1, 0, 0]), result);

	let (result, _) = U256([0, MAX, 0, 0]).overflowing_mul(U256([MAX, 0, 0, 0]));
	assert_eq!(U256([0, 1, MAX - 1, 0]), result);

	let (result, _) = U256([MAX, MAX, 0, 0]).overflowing_mul(U256([MAX, 0, 0, 0]));
	assert_eq!(U256([1, MAX, MAX - 1, 0]), result);

	let (result, _) = U256([MAX, 0, 0, 0]).overflowing_mul(U256([MAX, MAX, 0, 0]));
	assert_eq!(U256([1, MAX, MAX - 1, 0]), result);

	let (result, _) = U256([MAX, MAX, 0, 0]).overflowing_mul(U256([MAX, MAX, 0, 0]));
	assert_eq!(U256([1, 0, MAX - 1, MAX]), result);

	let (result, _) = U256([MAX, 0, 0, 0]).overflowing_mul(U256([MAX, MAX, MAX, 0]));
	assert_eq!(U256([1, MAX, MAX, MAX - 1]), result);

	let (result, _) = U256([MAX, MAX, MAX, 0]).overflowing_mul(U256([MAX, 0, 0, 0]));
	assert_eq!(U256([1, MAX, MAX, MAX - 1]), result);

	let (result, _) = U256([MAX, 0, 0, 0]).overflowing_mul(U256([MAX, MAX, MAX, MAX]));
	assert_eq!(U256([1, MAX, MAX, MAX]), result);

	let (result, _) = U256([MAX, MAX, MAX, MAX]).overflowing_mul(U256([MAX, 0, 0, 0]));
	assert_eq!(U256([1, MAX, MAX, MAX]), result);

	let (result, _) = U256([MAX, MAX, MAX, 0]).overflowing_mul(U256([MAX, MAX, 0, 0]));
	assert_eq!(U256([1, 0, MAX, MAX - 1]), result);

	let (result, _) = U256([MAX, MAX, 0, 0]).overflowing_mul(U256([MAX, MAX, MAX, 0]));
	assert_eq!(U256([1, 0, MAX, MAX - 1]), result);

	let (result, _) = U256([MAX, MAX, MAX, MAX]).overflowing_mul(U256([MAX, MAX, 0, 0]));
	assert_eq!(U256([1, 0, MAX, MAX]), result);

	let (result, _) = U256([MAX, MAX, 0, 0]).overflowing_mul(U256([MAX, MAX, MAX, MAX]));
	assert_eq!(U256([1, 0, MAX, MAX]), result);

	let (result, _) = U256([MAX, MAX, MAX, 0]).overflowing_mul(U256([MAX, MAX, MAX, 0]));
	assert_eq!(U256([1, 0, 0, MAX - 1]), result);

	let (result, _) = U256([MAX, MAX, MAX, 0]).overflowing_mul(U256([MAX, MAX, MAX, MAX]));
	assert_eq!(U256([1, 0, 0, MAX]), result);

	let (result, _) = U256([MAX, MAX, MAX, MAX]).overflowing_mul(U256([MAX, MAX, MAX, 0]));
	assert_eq!(U256([1, 0, 0, MAX]), result);

	let (result, _) = U256([0, 0, 0, MAX]).overflowing_mul(U256([0, 0, 0, MAX]));
	assert_eq!(U256([0, 0, 0, 0]), result);

	let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, MAX]));
	assert_eq!(U256([0, 0, 0, MAX]), result);

	let (result, _) = U256([MAX, MAX, MAX, MAX]).overflowing_mul(U256([MAX, MAX, MAX, MAX]));
	assert_eq!(U256([1, 0, 0, 0]), result);
}

#[test]
fn u256_multi_muls() {
	let (result, _) = U256([0, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
	assert_eq!(U256([0, 0, 0, 0]), result);

	let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([1, 0, 0, 0]));
	assert_eq!(U256([1, 0, 0, 0]), result);

	let (result, _) = U256([5, 0, 0, 0]).overflowing_mul(U256([5, 0, 0, 0]));
	assert_eq!(U256([25, 0, 0, 0]), result);

	let (result, _) = U256([0, 5, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
	assert_eq!(U256([0, 0, 25, 0]), result);

	let (result, _) = U256([0, 0, 0, 1]).overflowing_mul(U256([1, 0, 0, 0]));
	assert_eq!(U256([0, 0, 0, 1]), result);

	let (result, _) = U256([0, 0, 0, 5]).overflowing_mul(U256([2, 0, 0, 0]));
	assert_eq!(U256([0, 0, 0, 10]), result);

	let (result, _) = U256([0, 0, 1, 0]).overflowing_mul(U256([0, 5, 0, 0]));
	assert_eq!(U256([0, 0, 0, 5]), result);

	let (result, _) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
	assert_eq!(U256([0, 0, 0, 0]), result);

	let (result, _) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
	assert_eq!(U256([0, 10, 0, 0]), result);

	let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, MAX]));
	assert_eq!(U256([0, 0, 0, MAX]), result);
}

#[test]
fn u256_multi_muls_overflow() {
	let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
	assert!(!overflow);

	let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, MAX]));
	assert!(!overflow);

	let (_, overflow) = U256([0, 1, 0, 0]).overflowing_mul(U256([0, 0, 0, MAX]));
	assert!(overflow);

	let (_, overflow) = U256([0, 1, 0, 0]).overflowing_mul(U256([0, 1, 0, 0]));
	assert!(!overflow);

	let (_, overflow) = U256([0, 1, 0, MAX]).overflowing_mul(U256([0, 1, 0, MAX]));
	assert!(overflow);

	let (_, overflow) = U256([0, MAX, 0, 0]).overflowing_mul(U256([0, MAX, 0, 0]));
	assert!(!overflow);

	let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([10, 0, 0, 0]));
	assert!(!overflow);

	let (_, overflow) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, MAX / 2]));
	assert!(!overflow);

	let (_, overflow) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
	assert!(overflow);
}

#[test]
fn u512_div() {
	let fuzz_data = [
		0x38, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0x7, 0x0, 0x0, 0x0, 0x0, 0xc1,
		0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
		0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
		0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xfe, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
		0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
		0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
	];
	let a = U512::from_little_endian(&fuzz_data[..64]);
	let b = U512::from_little_endian(&fuzz_data[64..]);
	let (x, y) = (a / b, a % b);
	let (q, r) = a.div_mod(b);
	assert_eq!((x, y), (q, r));
}

#[test]
fn big_endian() {
	let source = U256([1, 0, 0, 0]);
	let mut target = vec![0u8; 32];

	assert_eq!(source, U256::from(1));

	source.put_big_endian(&mut target);
	assert_eq!(
		vec![
			0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
			0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8
		],
		target
	);

	let source = U256([512, 0, 0, 0]);
	let mut target = vec![0u8; 32];

	source.put_big_endian(&mut target);
	assert_eq!(
		vec![
			0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
			0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8
		],
		target
	);

	let source = U256([0, 512, 0, 0]);
	let mut target = vec![0u8; 32];

	source.put_big_endian(&mut target);
	assert_eq!(
		vec![
			0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
			0u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8
		],
		target
	);

	let source = U256::from_str("455867356320691211509944977504407603390036387149619137164185182714736811808").unwrap();
	source.put_big_endian(&mut target);
	assert_eq!(
		vec![
			0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12,
			0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20
		],
		target
	);
}

#[test]
fn u256_multi_muls2() {
	let (result, _) = U256([0, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
	assert_eq!(U256([0, 0, 0, 0]), result);

	let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([1, 0, 0, 0]));
	assert_eq!(U256([1, 0, 0, 0]), result);

	let (result, _) = U256([5, 0, 0, 0]).overflowing_mul(U256([5, 0, 0, 0]));
	assert_eq!(U256([25, 0, 0, 0]), result);

	let (result, _) = U256([0, 5, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
	assert_eq!(U256([0, 0, 25, 0]), result);

	let (result, _) = U256([0, 0, 0, 1]).overflowing_mul(U256([1, 0, 0, 0]));
	assert_eq!(U256([0, 0, 0, 1]), result);

	let (result, _) = U256([0, 0, 0, 5]).overflowing_mul(U256([2, 0, 0, 0]));
	assert_eq!(U256([0, 0, 0, 10]), result);

	let (result, _) = U256([0, 0, 1, 0]).overflowing_mul(U256([0, 5, 0, 0]));
	assert_eq!(U256([0, 0, 0, 5]), result);

	let (result, _) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
	assert_eq!(U256([0, 0, 0, 0]), result);

	let (result, _) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
	assert_eq!(U256([0, 10, 0, 0]), result);

	let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, u64::max_value()]));
	assert_eq!(U256([0, 0, 0, u64::max_value()]), result);

	let x1: U256 = "1251531179555".into();
	let x2sqr_right: U256 = "1566330293398329649998025".into();
	let x1sqr = x1 * x1;
	assert_eq!(x2sqr_right, x1sqr);

	let x1cube = x1sqr * x1;
	let x1cube_right: U256 = "1960311199669540736328758531670378875".into();
	assert_eq!(x1cube_right, x1cube);

	let x1quad = x1cube * x1;
	let x1quad_right: U256 = "2453390588017297443942654405410199097882503900625".into();
	assert_eq!(x1quad_right, x1quad);

	let x1penta = x1quad * x1;
	let x1penta_right: U256 = "3070494816530423318760836757780743650600287009546094751721875".into();
	assert_eq!(x1penta_right, x1penta);

	let x1septima = x1penta * x1;
	let x1septima_right: U256 = "3842819999549834008672227788404135925100853984878767509766273086046265625".into();
	assert_eq!(x1septima_right, x1septima);
}

#[test]
fn example() {
	let mut val: U256 = 1023.into();
	for _ in 0..200 {
		val = val * U256::from(2)
	}
	assert_eq!(&format!("{}", val), "1643897619276947051879427220465009342380213662639797070513307648");
}

#[test]
fn little_endian() {
	let number: U256 = "3842819999549834008672227788404135925100853984878767509766273086046265625".into();
	let expected = [
		0x19, 0xe1, 0xee, 0xd7, 0x32, 0xf9, 0x42, 0x30, 0x29, 0xdd, 0x08, 0xa7, 0xc5, 0xeb, 0x65, 0x64, 0x48, 0xfb,
		0xbb, 0xc5, 0x3c, 0x7d, 0x2b, 0x72, 0xe5, 0xf6, 0xa3, 0x1d, 0xca, 0x2c, 0x02, 0x00,
	];
	let mut result = [0u8; 32];
	number.put_little_endian(&mut result);
	assert_eq!(expected, result);
}

#[test]
fn slice_roundtrip() {
	let raw = [
		1u8, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103,
		107, 109, 113, 127,
	];

	let u256: U256 = U256::from_big_endian(&raw[..]);

	let mut new_raw = [0u8; 32];

	u256.put_big_endian(&mut new_raw);

	assert_eq!(&raw, &new_raw);
}

#[test]
fn slice_roundtrip_le() {
	let raw = [
		1u8, 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103,
		107, 109, 113, 127,
	];

	let u256 = U256::from_little_endian(&raw[..]);

	let mut new_raw = [0u8; 32];

	u256.put_little_endian(&mut new_raw);

	assert_eq!(&raw, &new_raw);
}

#[test]
fn slice_roundtrip_le2() {
	let raw = [
		2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107,
		109, 113, 127,
	];

	let u256 = U256::from_little_endian(&raw[..]);

	let mut new_raw = [0u8; 32];

	u256.put_little_endian(&mut new_raw);

	assert_eq!(&raw, &new_raw[..31]);
}

#[test]
fn from_little_endian() {
	let source: [u8; 32] =
		[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

	let number = U256::from_little_endian(&source[..]);

	assert_eq!(U256::from(1), number);
}

#[test]
fn from_big_endian() {
	let source: [u8; 32] =
		[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

	let number = U256::from_big_endian(&source[..]);

	assert_eq!(U256::from(1), number);

	let number = U256::from_big_endian(&[]);
	assert_eq!(U256::ZERO, number);

	let number = U256::from_big_endian(&[1]);
	assert_eq!(U256::from(1), number);
}

#[test]
fn into_fixed_array() {
	let expected = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
	let ary = U256::from(1).to_big_endian();
	assert_eq!(ary, expected);
}

#[test]
fn test_u256_from_fixed_array() {
	let ary = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 123];
	let num: U256 = U256::from_big_endian(&ary);
	assert_eq!(num, U256::from(core::u64::MAX) + U256::from(1 + 123) );

	let a_ref: &U256 = &U256::from_big_endian(&ary);
	assert_eq!(a_ref, &(U256::from(core::u64::MAX) + U256::from(1 + 123) ));
}

#[test]
fn test_from_ref_to_fixed_array() {
	let ary: &[u8; 32] =
		&[1, 0, 1, 2, 1, 0, 1, 2, 3, 0, 3, 4, 3, 0, 3, 4, 5, 0, 5, 6, 5, 0, 5, 6, 7, 0, 7, 8, 7, 0, 7, 8];
	let big: U256 = U256::from_big_endian(ary);
	// the numbers are each row of 8 bytes reversed and cast to u64
	assert_eq!(big, U256([504410889324070664, 360293493601469702, 216176097878868740, 72058702156267778u64]));
}

#[test]
fn test_u512_from_fixed_array() {
	let ary = [
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123,
	];
	let num: U512 = U512::from_big_endian(&ary);
	assert_eq!(num, U512::from(123));

	let a_ref: &U512 = &U512::from_big_endian(&ary);
	assert_eq!(a_ref, &U512::from(123));
}

#[test]
fn leading_zeros() {
	assert_eq!(U256::from("2453390588017297443942654405410199097882503900625").leading_zeros(), 95);
	assert_eq!(U256::from("108555083659983933209597798448098304200457908067980683192401684105301062938065").leading_zeros(), 0);
	assert_eq!(U256::from("1").leading_zeros(), 255);
	assert_eq!(U256::from("0").leading_zeros(), 256);
}

#[test]
fn trailing_zeros() {
	assert_eq!(U256::from("12148601763650061643489081113235337198383296159481012291642150536305704960000").trailing_zeros(), 92);
	assert_eq!(U256::from("12148601763650061643489081113235337198383296159481012291642150536305704960015").trailing_zeros(), 0);
	assert_eq!(U256::from("57896044618658097711785492504343953926634992332820282019728792003956564819968").trailing_zeros(), 255);
	assert_eq!(U256::from("0").trailing_zeros(), 256);
}

#[cfg(feature = "quickcheck")]
pub mod laws {
	use super::construct_uint;
	macro_rules! uint_laws {
		($mod_name:ident, $uint_ty:ident) => {
			mod $mod_name {
				use qc::{TestResult, quickcheck};
				use super::$uint_ty;

				quickcheck! {
					fn associative_add(x: $uint_ty, y: $uint_ty, z: $uint_ty) -> TestResult {
						if x.overflowing_add(y).1 || y.overflowing_add(z).1 || (x + y).overflowing_add(z).1 {
							return TestResult::discard();
						}

						TestResult::from_bool(
							(x + y) + z == x + (y + z)
						)
					}
				}

				quickcheck! {
					fn associative_mul(x: $uint_ty, y: $uint_ty, z: $uint_ty) -> TestResult {
						if x.overflowing_mul(y).1 || y.overflowing_mul(z).1 || (x * y).overflowing_mul(z).1 {
							return TestResult::discard();
						}

						TestResult::from_bool(
							(x * y) * z == x * (y * z)
						)
					}
				}

				quickcheck! {
					fn commutative_add(x: $uint_ty, y: $uint_ty) -> TestResult {
						if x.overflowing_add(y).1 {
							return TestResult::discard();
						}

						TestResult::from_bool(
							x + y == y + x
						)
					}
				}

				quickcheck! {
					fn commutative_mul(x: $uint_ty, y: $uint_ty) -> TestResult {
						if x.overflowing_mul(y).1 {
							return TestResult::discard();
						}

						TestResult::from_bool(
							x * y == y * x
						)
					}
				}

				quickcheck! {
					fn identity_add(x: $uint_ty) -> bool {
						x + $uint_ty::ZERO == x
					}
				}

				quickcheck! {
					fn identity_mul(x: $uint_ty) -> bool {
						x * $uint_ty::one() == x
					}
				}

				quickcheck! {
					fn identity_div(x: $uint_ty) -> bool {
						x / $uint_ty::one() == x
					}
				}

				quickcheck! {
					fn absorbing_rem(x: $uint_ty) -> bool {
						x % $uint_ty::one() == $uint_ty::ZERO
					}
				}

				quickcheck! {
					fn absorbing_sub(x: $uint_ty) -> bool {
						x - x == $uint_ty::ZERO
					}
				}

				quickcheck! {
					fn absorbing_mul(x: $uint_ty) -> bool {
						x * $uint_ty::ZERO == $uint_ty::ZERO
					}
				}

				quickcheck! {
					fn distributive_mul_over_add(x: $uint_ty, y: $uint_ty, z: $uint_ty) -> TestResult {
						if y.overflowing_add(z).1 || x.overflowing_mul(y + z).1 || x.overflowing_add(y).1 || (x + y).overflowing_mul(z).1 {
							return TestResult::discard();
						}

						TestResult::from_bool(
							(x * (y + z) == (x * y + x * z)) && (((x + y) * z) == (x * z + y * z))
						)
					}
				}

				quickcheck! {
					fn pow_mul(x: $uint_ty) -> TestResult {
						if x.overflowing_pow($uint_ty::from(2)).1 || x.overflowing_pow($uint_ty::from(3)).1 {
							// On overflow `checked_pow` should return `None`.
							assert_eq!(x.checked_pow($uint_ty::from(2)), None);
							assert_eq!(x.checked_pow($uint_ty::from(3)), None);

							return TestResult::discard();
						}

						TestResult::from_bool(
							x.pow($uint_ty::from(2)) == x * x && x.pow($uint_ty::from(3)) == x * x * x
						)
					}
				}

				quickcheck! {
					fn add_increases(x: $uint_ty, y: $uint_ty) -> TestResult {
						if y.is_zero() || x.overflowing_add(y).1 {
							return TestResult::discard();
						}

						TestResult::from_bool(
							x + y > x
						)
					}
				}

				quickcheck! {
					fn mul_increases(x: $uint_ty, y: $uint_ty) -> TestResult {
						if y.is_zero() || x.overflowing_mul(y).1 {
							return TestResult::discard();
						}

						TestResult::from_bool(
							x * y >= x
						)
					}
				}

				quickcheck! {
					fn div_decreases_dividend(x: $uint_ty, y: $uint_ty) -> TestResult {
						if y.is_zero() {
							return TestResult::discard();
						}

						TestResult::from_bool(
							x / y <= x
						)
					}
				}

				quickcheck! {
					fn rem_decreases_divisor(x: $uint_ty, y: $uint_ty) -> TestResult {
						if y.is_zero() {
							return TestResult::discard();
						}

						TestResult::from_bool(
							x % y < y
						)
					}
				}
			}
		}
	}

	construct_uint! {
		pub struct U64(1);
	}
	construct_uint! {
		pub struct U256(4);
	}
	construct_uint! {
		pub struct U512(8);
	}
	construct_uint! {
		pub struct U1024(16);
	}

	uint_laws!(u64, U64);
	uint_laws!(u256, U256);
	uint_laws!(u512, U512);
	uint_laws!(u1024, U1024);
}
