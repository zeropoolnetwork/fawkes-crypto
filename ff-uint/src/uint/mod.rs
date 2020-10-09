#[macro_use]
pub(crate) mod macros;
pub(crate) mod traits;

#[macro_export]
macro_rules! construct_uint {
	($(#[$attr:meta])* $visibility:vis struct $name:ident ( $n_words:tt );) => {
		$crate::concat_idents!(wrapped_mod = wrapped_mod, _, $name {
			$visibility use wrapped_mod::$name;
			mod wrapped_mod {
				use $crate::unroll;
				use $crate::Uint;
				use $crate::borsh::{BorshSerialize, BorshDeserialize};

				#[repr(C)]
				$(#[$attr])*
				#[derive(Copy, Clone, Default, BorshSerialize, BorshDeserialize)]
				pub struct $name (pub [u64; $n_words]);

				#[inline]
				const fn uint_from_u64(v:u64) -> $name {
					let mut ret = [0;$n_words];
					ret[0]=v;
					$name(ret)
				}

				impl $name {
					const WORD_BITS:usize = 64;
					const NUM_WORDS : usize = $n_words;

					const MAX : Self = Self([u64::max_value(); $n_words]);
					const ZERO : Self = Self([0u64; $n_words]);
					const ONE : Self = uint_from_u64(1);

					#[inline]
					pub const fn new(v:[u64; $n_words]) -> Self {
						Self(v)
					}

					// Whether this fits u64.
					#[inline]
					fn fits_word(&self) -> bool {
						let &$name(ref arr) = self;
						for i in 1..$n_words { if arr[i] != 0 { return false; } }
						return true;
					}


					#[inline]
					fn full_shl(self, shift: u32) -> [u64; $n_words + 1] {
						debug_assert!(shift < Self::WORD_BITS as u32);
						let mut u = [064; $n_words + 1];
						let u_lo = self.0[0] << shift;
						let u_hi = self.overflowing_shr(Self::WORD_BITS as u32 - shift).0;
						u[0] = u_lo;
						u[1..].copy_from_slice(&u_hi.0[..]);
						u
					}

					#[inline]
					fn full_shr(u: [u64; $n_words + 1], shift: u32) -> Self {
						debug_assert!(shift < Self::WORD_BITS as u32);
						let mut res = Self::ZERO;
						for i in 0..$n_words {
							res.0[i] = u[i] >> shift;
						}
						// carry
						if shift > 0 {
							for i in 1..=$n_words {
								res.0[i - 1] |= u[i] << (Self::WORD_BITS as u32 - shift);
							}
						}
						res
					}

					#[inline]
					fn full_mul_u64(self, by: u64) -> [u64; $n_words + 1] {
						let (prod, carry) = self.overflowing_mul_u64(by);
						let mut res = [0u64; $n_words + 1];
						res[..$n_words].copy_from_slice(&prod.0[..]);
						res[$n_words] = carry;
						res
					}

					#[inline]
					fn div_mod_small(mut self, other: u64) -> (Self, Self) {
						let mut rem = 0u64;
						self.0.iter_mut().rev().for_each(|d| {
							let (q, r) = Self::div_mod_word(rem, *d, other);
							*d = q;
							rem = r;
						});
						(self, Self::from_u64(rem))
					}

					// See Knuth, TAOCP, Volume 2, section 4.3.1, Algorithm D.
					#[inline]
					fn div_mod_knuth(self, mut v: Self, n: usize, m: usize) -> (Self, Self) {
						debug_assert!(self.bits() >= v.bits() && !v.fits_word());
						debug_assert!(n + m <= $n_words);
						// D1.
						// Make sure 64th bit in v's highest word is set.
						// If we shift both self and v, it won't affect the quotient
						// and the remainder will only need to be shifted back.
						let shift = v.0[n - 1].leading_zeros();
						v = v.overflowing_shl(shift).0;
						// u will store the remainder (shifted)
						let mut u = self.full_shl(shift);

						// quotient
						let mut q = Self::ZERO;
						let v_n_1 = v.0[n - 1];
						let v_n_2 = v.0[n - 2];

						// D2. D7.
						// iterate from m downto 0
						for j in (0..=m).rev() {
							let u_jn = u[j + n];

							// D3.
							// q_hat is our guess for the j-th quotient digit
							// q_hat = min(b - 1, (u_{j+n} * b + u_{j+n-1}) / v_{n-1})
							// b = 1 << WORD_BITS
							// Theorem B: q_hat >= q_j >= q_hat - 2
							let mut q_hat = if u_jn < v_n_1 {
								let (mut q_hat, mut r_hat) = Self::div_mod_word(u_jn, u[j + n - 1], v_n_1);
								// this loop takes at most 2 iterations
								loop {
									// check if q_hat * v_{n-2} > b * r_hat + u_{j+n-2}
									let (hi, lo) = Self::split_u128(u128::from(q_hat) * u128::from(v_n_2));
									if (hi, lo) <= (r_hat, u[j + n - 2]) {
										break;
									}
									// then iterate till it doesn't hold
									q_hat -= 1;
									let (new_r_hat, overflow) = r_hat.overflowing_add(v_n_1);
									r_hat = new_r_hat;
									// if r_hat overflowed, we're done
									if overflow {
										break;
									}
								}
								q_hat
							} else {
								// here q_hat >= q_j >= q_hat - 1
								u64::max_value()
							};

							// ex. 20:
							// since q_hat * v_{n-2} <= b * r_hat + u_{j+n-2},
							// either q_hat == q_j, or q_hat == q_j + 1

							// D4.
							// let's assume optimistically q_hat == q_j
							// subtract (q_hat * v) from u[j..]
							let q_hat_v = v.full_mul_u64(q_hat);
							// u[j..] -= q_hat_v;
							let c = Self::sub_slice(&mut u[j..], &q_hat_v[..n + 1]);

							// D6.
							// actually, q_hat == q_j + 1 and u[j..] has overflowed
							// highly unlikely ~ (1 / 2^63)
							if c {
								q_hat -= 1;
								// add v to u[j..]
								let c = Self::add_slice(&mut u[j..], &v.0[..n]);
								u[j + n] = u[j + n].wrapping_add(u64::from(c));
							}

							// D5.
							q.0[j] = q_hat;
						}

						// D8.
						let remainder = Self::full_shr(u, shift);

						(q, remainder)
					}

					// Returns the least number of words needed to represent the nonzero number
					#[inline]
					fn words(bits: usize) -> usize {
						debug_assert!(bits > 0);
						(bits + Self::WORD_BITS - 1) / Self::WORD_BITS
					}

					#[inline(always)]
					fn div_mod_word(hi: u64, lo: u64, y: u64) -> (u64, u64) {
						debug_assert!(hi < y);
						// NOTE: this is slow (__udivti3)
						// let x = (u128::from(hi) << 64) + u128::from(lo);
						// let d = u128::from(d);
						// ((x / d) as u64, (x % d) as u64)
						// TODO: look at https://gmplib.org/~tege/division-paper.pdf
						const TWO32: u64 = 1 << 32;
						let s = y.leading_zeros();
						let y = y << s;
						let (yn1, yn0) = Self::split(y);
						let un32 = (hi << s) | lo.checked_shr(64 - s).unwrap_or(0);
						let un10 = lo << s;
						let (un1, un0) = Self::split(un10);
						let mut q1 = un32.wrapping_div(yn1);
						let mut rhat = un32 - q1 * yn1;

						while q1 >= TWO32 || q1 * yn0 > TWO32 * rhat + un1 {
							q1 -= 1;
							rhat += yn1;
							if rhat >= TWO32 {
								break;
							}
						}

						let un21 = un32.wrapping_mul(TWO32).wrapping_add(un1).wrapping_sub(q1.wrapping_mul(y));
						let mut q0 = un21.wrapping_div(yn1);
						rhat = un21.wrapping_sub(q0.wrapping_mul(yn1));

						while q0 >= TWO32 || q0 * yn0 > TWO32 * rhat + un0 {
							q0 -= 1;
							rhat += yn1;
							if rhat >= TWO32 {
								break;
							}
						}

						let rem = un21.wrapping_mul(TWO32).wrapping_add(un0).wrapping_sub(y.wrapping_mul(q0));
						(q1 * TWO32 + q0, rem >> s)
					}

					#[inline]
					fn add_slice(a: &mut [u64], b: &[u64]) -> bool {
						Self::binop_slice(a, b, u64::overflowing_add)
					}

					#[inline]
					fn sub_slice(a: &mut [u64], b: &[u64]) -> bool {
						Self::binop_slice(a, b, u64::overflowing_sub)
					}

					#[inline]
					fn binop_slice(a: &mut [u64], b: &[u64], binop: impl Fn(u64, u64) -> (u64, bool) + Copy) -> bool {
						let mut c = false;
						a.iter_mut().zip(b.iter()).for_each(|(x, y)| {
							let (res, carry) = Self::binop_carry(*x, *y, c, binop);
							*x = res;
							c = carry;
						});
						c
					}

					#[inline]
					fn binop_carry(a: u64, b: u64, c: bool, binop: impl Fn(u64, u64) -> (u64, bool)) -> (u64, bool) {
						let (res1, overflow1) = b.overflowing_add(u64::from(c));
						let (res2, overflow2) = binop(a, res1);
						(res2, overflow1 || overflow2)
					}

					#[inline]
					const fn mul_u64(a: u64, b: u64, carry: u64) -> (u64, u64) {
						let (hi, lo) = Self::split_u128(a as u128 * b as u128 + carry as u128);
						(lo, hi)
					}

					#[inline]
					const fn split(a: u64) -> (u64, u64) {
						(a >> 32, a & 0xFFFF_FFFF)
					}

					#[inline]
					const fn split_u128(a: u128) -> (u64, u64) {
						((a >> 64) as _, (a & 0xFFFFFFFFFFFFFFFF) as _)
					}

				}

				$crate::impl_map_from!($name, bool, u64);
				$crate::impl_map_from!($name, u8, u64);
				$crate::impl_map_from!($name, u16, u64);
				$crate::impl_map_from!($name, u32, u64);
				$crate::impl_map_from!($name, usize, u64);
				$crate::impl_map_from!($name, i8, i64);
				$crate::impl_map_from!($name, i16, i64);
				$crate::impl_map_from!($name, i32, i64);
				$crate::impl_map_from!($name, isize, i64);


				impl std::convert::TryFrom<$name> for bool {
					type Error = &'static str;

					#[inline]
					fn try_from(u: $name) -> std::result::Result<bool, &'static str> {
						let $name(arr) = u;
						if !u.fits_word() || arr[0] > 2 {
							Err("integer overflow when casting to bool",)
						} else {
							Ok(arr[0] == 1)
						}
					}
				}

				$crate::impl_try_from_for_primitive!($name, u8);
				$crate::impl_try_from_for_primitive!($name, u16);
				$crate::impl_try_from_for_primitive!($name, u32);
				$crate::impl_try_from_for_primitive!($name, usize);
				$crate::impl_try_from_for_primitive!($name, u64);
				$crate::impl_try_from_for_primitive!($name, i8);
				$crate::impl_try_from_for_primitive!($name, i16);
				$crate::impl_try_from_for_primitive!($name, i32);
				$crate::impl_try_from_for_primitive!($name, isize);
				$crate::impl_try_from_for_primitive!($name, i64);



				$crate::impl_typecast_128!($name, $n_words);

				impl std::cmp::Ord for $name {
					#[inline]
					fn cmp(&self, other: &$name) -> std::cmp::Ordering {
						self.wrapping_cmp(other)
					}
				}

				impl std::cmp::PartialOrd for $name {
					#[inline]
					fn partial_cmp(&self, other: &$name) -> Option<std::cmp::Ordering> {
						Some(self.cmp(other))
					}
				}

				impl std::cmp::PartialEq for $name {
					#[inline]
					fn eq(&self, other: &$name) -> bool {
						let &$name(ref a) = self;
						let &$name(ref b) = other;
						unroll!{
							for i in 0 .. $n_words {
								if a[i] != b[i] { return false; }
							}
						}
						true
					}
				}

				impl std::cmp::Eq for $name {}

				impl std::hash::Hash for $name {
					fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
						self.as_inner().as_ref().hash(state);
					}
				}

				impl std::fmt::Debug for $name {
					fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
						std::fmt::Display::fmt(self, f)
					}
				}

				impl std::fmt::Display for $name {
					fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
						if self.is_zero() {
							return std::write!(f, "0");
						}

						let mut buf = [0_u8; $n_words*20];
						let mut i = buf.len() - 1;
						let mut current = *self;
						let ten = uint_from_u64(10);
						loop {
							let t = current.wrapping_rem(ten);
							let digit = t.low_u64() as u8;
							buf[i] = digit + b'0';
							current = current.wrapping_div(ten);
							if current.is_zero() {
								break;
							}
							i -= 1;
						}

						// sequence of `'0'..'9'` chars is guaranteed to be a valid UTF8 string
						let s = unsafe {
							std::str::from_utf8_unchecked(&buf[i..])
						};
						f.write_str(s)
					}
				}

				impl std::fmt::LowerHex for $name {
					fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
						let &$name(ref data) = self;
						if f.alternate() {
							std::write!(f, "0x")?;
						}
						// special case.
						if self.is_zero() {
							return std::write!(f, "0");
						}

						let mut latch = false;
						for ch in data.iter().rev() {
							for x in 0..16 {
								let nibble = (ch & (15u64 << ((15 - x) * 4) as u64)) >> (((15 - x) * 4) as u64);
								if !latch {
									latch = nibble != 0;
								}

								if latch {
									std::write!(f, "{:x}", nibble)?;
								}
							}
						}
						Ok(())
					}
				}

				impl $crate::rand::distributions::Distribution<$name> for $crate::rand::distributions::Standard {
					#[inline]
					fn sample<R: $crate::rand::Rng + ?Sized>(&self, rng: &mut R) -> $name {
						$name(rng.gen())
					}
				}

				impl std::str::FromStr for $name {
					type Err = &'static str;

					fn from_str(value: &str) -> std::result::Result<$name, Self::Err> {
						if !value.bytes().all(|b| b >= 48 && b <= 57) {
							return Err("Invalid character")
						}

						let mut res = Self::default();
						for b in value.bytes().map(|b| b - 48) {
							let (r, overflow) = res.overflowing_mul_u64(10);
							if overflow > 0 {
								return Err("Invalid length");
							}
							let (r, overflow) = r.overflowing_add(uint_from_u64(b as u64));
							if overflow {
								return Err("Invalid length");
							}
							res = r;
						}
						Ok(res)
					}
				}

				impl std::convert::From<&'static str> for $name {
					fn from(s: &'static str) -> Self {
						s.parse().unwrap()
					}
				}

				impl std::convert::From<u64> for $name {
					fn from(value: u64) -> $name {
						let mut ret = [0; $n_words];
						ret[0] = value;
						$name(ret)
					}
				}

				impl std::convert::From<i64> for $name {
					fn from(value: i64) -> $name {
						match value >= 0 {
							true => From::from(value as u64),
							false => { panic!("Unsigned integer can't be created from negative value"); }
						}
					}
				}



				impl Uint for $name {
					type Inner = [u64; $n_words];

					const MAX : Self = Self::MAX;
					const ZERO : Self = Self::ZERO;
					const ONE : Self = Self::ONE;

					const NUM_WORDS : usize = Self::NUM_WORDS;
					const WORD_BITS : usize = Self::WORD_BITS;

					#[inline]
					fn random<R: rand::Rng + ?Sized>(rng: &mut R) -> Self {
						rng.gen()
					}

					#[inline]
					fn into_inner(self) -> Self::Inner {
						self.0
					}

					#[inline]
					fn as_inner(&self) -> &Self::Inner {
						let &Self(ref res) = self;
						res
					}

					#[inline]
					fn as_inner_mut(&mut self) -> &mut Self::Inner {
						let &mut Self(ref mut res) = self;
						res
					}

					#[inline]
					fn as_u64(&self) -> u64 {
						if !self.fits_word() {
							panic!("Integer overflow when casting to u64")
						} else {
							self.low_u64()
						}
					}

					#[inline]
					fn low_u64(&self) -> u64 {
						self.0[0]
					}

					#[inline]
					fn from_u64(v:u64) -> Self {
						uint_from_u64(v)
					}

					/// Whether this is zero.
					#[inline]
					fn is_zero(&self) -> bool {
						let &Self(ref arr) = self;
						for i in 0..$n_words { if arr[i] != 0 { return false; } }
						return true;
					}

					/// Returns the number of leading zeros in the binary representation of self.
					#[inline]
					fn leading_zeros(&self) -> u32 {
						let mut r = 0;
						for i in 0..$n_words {
							let w = self.0[$n_words - i - 1];
							if w == 0 {
								r += 64;
							} else {
								r += w.leading_zeros();
								break;
							}
						}
						r
					}

					/// Returns the number of leading zeros in the binary representation of self.
					#[inline]
					fn trailing_zeros(&self) -> u32 {
						let mut r = 0;
						for i in 0..$n_words {
							let w = self.0[i];
							if w == 0 {
								r += 64;
							} else {
								r += w.trailing_zeros();
								break;
							}
						}
						r
					}

					#[inline]
					fn bits(&self) -> usize {
						let &Self(ref arr) = self;
						for i in 1..$n_words {
							if arr[$n_words - i] > 0 { return (0x40 * ($n_words - i + 1)) - arr[$n_words - i].leading_zeros() as usize; }
						}
						0x40 - arr[0].leading_zeros() as usize
					}


					/// Returns a pair `(self / other, self % other)`.
					///
					/// # Panics
					///
					/// Panics if `other` is zero.
					#[inline]
					fn div_mod(mut self, mut other: Self) -> (Self, Self) {
						use std::cmp::Ordering;

						let my_bits = self.bits();
						let your_bits = other.bits();

						assert!(your_bits != 0, "division by zero");

						// Early return in case we are dividing by a larger number than us
						if my_bits < your_bits {
							return (Self::ZERO, self);
						}

						if your_bits <= Self::WORD_BITS {
							return self.div_mod_small(other.low_u64());
						}

						let (n, m) = {
							let my_words = Self::words(my_bits);
							let your_words = Self::words(your_bits);
							(your_words, my_words - your_words)
						};

						self.div_mod_knuth(other, n, m)
					}

					#[inline]
					fn overflowing_add(self, other: Self) -> (Self, bool) {
						$crate::uint_overflowing_binop!(
							$name,
							$n_words,
							self,
							other,
							u64::overflowing_add
						)
					}

					#[inline]
					fn overflowing_sub(self, other: Self) -> (Self, bool) {
						$crate::uint_overflowing_binop!(
							$name,
							$n_words,
							self,
							other,
							u64::overflowing_sub
						)
					}

					/// Overflowing multiplication by u64.
					/// Returns the result and carry.
					#[inline]
					fn overflowing_mul_u64(mut self, other: u64) -> (Self, u64) {
						let mut carry = 0u64;

						for d in self.0.iter_mut() {
							let (res, c) = Self::mul_u64(*d, other, carry);
							*d = res;
							carry = c;
						}

						(self, carry)
					}

					#[inline]
					fn overflowing_mul(self, other: Self) -> (Self, bool) {
						$crate::uint_overflowing_mul!($name, $n_words, self, other)
					}

					#[inline]
					fn overflowing_neg(self) -> (Self, bool) {
						(self.overflowing_not().0.wrapping_add(Self::from_u64(1)), !self.is_zero())
					}

					#[inline]
					fn overflowing_shl(self, lhs: u32) -> (Self, bool) {
						let shift = lhs as usize;
						let $name(ref original) = self;
						let mut ret = [0u64; $n_words];
						let word_shift = shift / 64;
						let bit_shift = shift % 64;

						// shift
						for i in word_shift..$n_words {
							ret[i] = original[i - word_shift] << bit_shift;
						}
						// carry
						if bit_shift > 0 {
							for i in word_shift+1..$n_words {
								ret[i] += original[i - 1 - word_shift] >> (64 - bit_shift);
							}
						}
						($name(ret), (lhs > ($n_words*64 - 1)))
					}

					#[inline]
					fn overflowing_shr(self, rhs: u32) -> ($name, bool) {
						let shift = rhs as usize;
						let $name(ref original) = self;
						let mut ret = [0u64; $n_words];
						let word_shift = shift / 64;
						let bit_shift = shift % 64;

						// shift
						for i in word_shift..$n_words {
							ret[i - word_shift] = original[i] >> bit_shift;
						}

						// Carry
						if bit_shift > 0 {
							for i in word_shift+1..$n_words {
								ret[i - word_shift - 1] += original[i] << (64 - bit_shift);
							}
						}
						($name(ret), (rhs > ($n_words*64 - 1)))
					}

					#[inline]
					fn overflowing_not(self) -> (Self, bool) {
						let mut ret = [0u64; $n_words];
						$crate::unroll! {
							for i in 0..$n_words {
								ret[i]=!self.0[i];
							}
						}
						(Self(ret), false)
					}

					#[inline]
					fn overflowing_bitand(self, other:Self) -> (Self, bool) {
						let mut ret = [0u64; $n_words];
						$crate::unroll! {
							for i in 0..$n_words {
								ret[i]=self.0[i] & other.0[i];
							}
						}
						(Self(ret), false)
					}

					#[inline]
					fn overflowing_bitor(self, other:Self) -> (Self, bool) {
						let mut ret = [0u64; $n_words];
						$crate::unroll! {
							for i in 0..$n_words {
								ret[i]=self.0[i] | other.0[i];
							}
						}
						(Self(ret), false)
					}

					#[inline]
					fn overflowing_bitxor(self, other:Self) -> (Self, bool) {
						let mut ret = [0u64; $n_words];
						$crate::unroll! {
							for i in 0..$n_words {
								ret[i]=self.0[i] ^ other.0[i];
							}
						}
						(Self(ret), false)
					}



					/// Write to the slice in big-endian format.
					#[inline]
					fn put_big_endian(&self, bytes: &mut [u8]) {
						use $crate::byteorder::{ByteOrder, BigEndian};
						debug_assert!($n_words * 8 == bytes.len());
						for i in 0..$n_words {
							BigEndian::write_u64(&mut bytes[8 * i..], self.0[$n_words - i - 1]);
						}
					}

					/// Write to the slice in little-endian format.
					#[inline]
					fn put_little_endian(&self, bytes: &mut [u8]) {
						use $crate::byteorder::{ByteOrder, LittleEndian};
						debug_assert!($n_words * 8 == bytes.len());
						for i in 0..$n_words {
							LittleEndian::write_u64(&mut bytes[8 * i..], self.0[i]);
						}
					}

					#[inline]
					fn to_big_endian(&self) -> Vec<u8> {
						let mut res = vec![0u8;$n_words*8];
						self.put_big_endian(&mut res);
						res
					}

					#[inline]
					fn to_little_endian(&self) -> Vec<u8> {
						let mut res = vec![0u8;$n_words*8];
						self.put_little_endian(&mut res);
						res
					}

					#[inline]
					fn from_big_endian(slice: &[u8]) -> Self {
						use $crate::byteorder::{ByteOrder, BigEndian};
						assert!($n_words * 8 >= slice.len());

						let mut padded = [0u8; $n_words * 8];
						padded[$n_words * 8 - slice.len() .. $n_words * 8].copy_from_slice(&slice);

						let mut ret = [0; $n_words];
						for i in 0..$n_words {
							ret[$n_words - i - 1] = BigEndian::read_u64(&padded[8 * i..]);
						}

						$name(ret)
					}

					#[inline]
					fn from_little_endian(slice: &[u8]) -> Self {
						use $crate::byteorder::{ByteOrder, LittleEndian};
						assert!($n_words * 8 >= slice.len());

						let mut padded = [0u8; $n_words * 8];
						padded[0..slice.len()].copy_from_slice(&slice);

						let mut ret = [0; $n_words];
						for i in 0..$n_words {
							ret[i] = LittleEndian::read_u64(&padded[8 * i..]);
						}

						$name(ret)
					}

					#[inline]
					fn wrapping_cmp(&self, other: &$name) -> std::cmp::Ordering {
						let &$name(ref a) = self;
						let &$name(ref b) = other;
						let mut i = $n_words;

						for i in (0 .. $n_words).rev() {
							if a[i] < b[i] { return std::cmp::Ordering::Less; }
							if a[i] > b[i] { return std::cmp::Ordering::Greater; }
						}

						std::cmp::Ordering::Equal
					}


				}
			}
		});
	};
}
