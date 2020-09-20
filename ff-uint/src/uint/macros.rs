#![allow(unused_macros)]


#[macro_export]
#[doc(hidden)]
macro_rules! uint_overflowing_binop {
	($name:ident, $n_words: tt, $self_expr: expr, $other: expr, $fn:expr) => {{
		let $name(ref me) = $self_expr;
		let $name(ref you) = $other;

		let mut ret = [0u64; $n_words];
		let ret_ptr = &mut ret as *mut [u64; $n_words] as *mut u64;
		let mut carry = 0u64;
		$crate::static_assertions::const_assert!(
			std::isize::MAX as usize / std::mem::size_of::<u64>() > $n_words
		);

		// `unroll!` is recursive, but doesn’t use `$crate::unroll`, so we need to ensure that it
		// is in scope unqualified.
		use $crate::unroll;
		unroll! {
			for i in 0..$n_words {
				use std::ptr;

				if carry != 0 {
					let (res1, overflow1) = ($fn)(me[i], you[i]);
					let (res2, overflow2) = ($fn)(res1, carry);

					unsafe {
						// SAFETY: `i` is within bounds and `i * size_of::<u64>() < isize::MAX`
						*ret_ptr.offset(i as _) = res2
					}
					carry = (overflow1 as u8 + overflow2 as u8) as u64;
				} else {
					let (res, overflow) = ($fn)(me[i], you[i]);

					unsafe {
						// SAFETY: `i` is within bounds and `i * size_of::<u64>() < isize::MAX`
						*ret_ptr.offset(i as _) = res
					}

					carry = overflow as u64;
				}
			}
		}

		($name(ret), carry > 0)
		}};
}

#[macro_export]
#[doc(hidden)]
macro_rules! uint_full_mul_reg {
	($name:ident, 8, $self_expr:expr, $other:expr) => {
		$crate::uint_full_mul_reg!($name, 8, $self_expr, $other, |a, b| a != 0 || b != 0);
	};
	($name:ident, $n_words:tt, $self_expr:expr, $other:expr) => {
		$crate::uint_full_mul_reg!($name, $n_words, $self_expr, $other, |_, _| true);
	};
	($name:ident, $n_words:tt, $self_expr:expr, $other:expr, $check:expr) => {{
		{
			#![allow(unused_assignments)]

			let $name(ref me) = $self_expr;
			let $name(ref you) = $other;
			let mut ret = [0u64; $n_words * 2];

			use $crate::unroll;
			unroll! {
				for i in 0..$n_words {
					let mut carry = 0u64;
					let b = you[i];

					unroll! {
						for j in 0..$n_words {
							if $check(me[j], carry) {
								let a = me[j];

								let (hi, low) = Self::split_u128(a as u128 * b as u128);

								let overflow = {
									let existing_low = &mut ret[i + j];
									let (low, o) = low.overflowing_add(*existing_low);
									*existing_low = low;
									o
								};

								carry = {
									let existing_hi = &mut ret[i + j + 1];
									let hi = hi + overflow as u64;
									let (hi, o0) = hi.overflowing_add(carry);
									let (hi, o1) = hi.overflowing_add(*existing_hi);
									*existing_hi = hi;

									(o0 | o1) as u64
								}
							}
						}
					}
				}
			}

			ret
		}
	}};
}

#[macro_export]
#[doc(hidden)]
macro_rules! uint_overflowing_mul {
	($name:ident, $n_words: tt, $self_expr: expr, $other: expr) => {{
		let ret: [u64; $n_words * 2] =
			$crate::uint_full_mul_reg!($name, $n_words, $self_expr, $other);

		// The safety of this is enforced by the compiler
		let ret: [[u64; $n_words]; 2] = unsafe { std::mem::transmute(ret) };

		// The compiler WILL NOT inline this if you remove this annotation.
		#[inline(always)]
		fn any_nonzero(arr: &[u64; $n_words]) -> bool {
			use $crate::unroll;
			unroll! {
				for i in 0..$n_words {
					if arr[i] != 0 {
						return true;
					}
				}
			}

			false
		}

		($name(ret[0]), any_nonzero(&ret[1]))
	}};
}



#[macro_export]
#[doc(hidden)]
macro_rules! overflowing {
	($op: expr, $overflow: expr) => {{
		let (overflow_x, overflow_overflow) = $op;
		$overflow |= overflow_overflow;
		overflow_x
	}};
	($op: expr) => {{
		let (overflow_x, _overflow_overflow) = $op;
		overflow_x
	}};
}

#[macro_export]
#[doc(hidden)]
macro_rules! panic_on_overflow {
	($name: expr) => {
		if $name as u64 != 0 {
			panic!("arithmetic operation overflow")
		}
	};
}




#[macro_export]
#[doc(hidden)]
macro_rules! impl_overflowing_binop{
    ($op: ident, $method:ident, $overflowing_op: ident, $name: ty) => {
        $crate::impl_overflowing_binop!($op, $method, $overflowing_op, $name, $name);
    };

	($op: ident, $method:ident, $overflowing_op: ident, $name: ty, $other: ty) => {
		impl std::ops::$op<$other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                let (res, overflow) = self.$overflowing_op(other);
                $crate::panic_on_overflow!(overflow);
                res
            }       
        }

		impl<'a> std::ops::$op<&'a $other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'a $other) -> Self::Output {
                let (res, overflow) = self.$overflowing_op(*other);
                $crate::panic_on_overflow!(overflow);
                res
            }       
        }   
    
		impl<'a> std::ops::$op<$other> for &'a $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                let (res, overflow) = (*self).$overflowing_op(other);
                $crate::panic_on_overflow!(overflow);
                res
            }       
        }

		impl<'a, 'b> std::ops::$op<&'a $other> for &'b $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'a $other) -> Self::Output {
                let (res, overflow) = (*self).$overflowing_op(*other);
                $crate::panic_on_overflow!(overflow);
                res
            }       
        }
	};
}


#[macro_export]
#[doc(hidden)]
macro_rules! impl_overflowing_unop {
	($op: ident, $method:ident, $overflowing_op: ident, $name: ty) => {
        impl std::ops::$op for $name {
            type Output = $name;
            #[inline]
            fn $method(self) -> $name {
                let (res, overflow) = self.$overflowing_op();
                $crate::panic_on_overflow!(overflow);
                res
            }
        }

        impl<'a> std::ops::$op for &'a $name {
            type Output = $name;
            #[inline]
            fn $method(self) -> $name {
                let (res, overflow) = (*self).$overflowing_op();
                $crate::panic_on_overflow!(overflow);
                res
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_overflowing_assignop{
    ($op: ident, $method:ident, $overflowing_op: ident, $name: ty) => {
        $crate::impl_overflowing_assignop!($op, $method, $overflowing_op, $name, $name);
    };


	($op: ident, $method:ident, $overflowing_op: ident, $name: ty, $other: ty) => {
		impl std::ops::$op<$other> for $name {
            #[inline]
			fn $method(&mut self, other: $other) {
                let (res, overflow) = (*self).$overflowing_op(other);
                $crate::panic_on_overflow!(overflow);
                *self = res;
            }       
        }

		impl<'a> std::ops::$op<&'a $other> for $name {
            #[inline]
			fn $method(&mut self, other: &'a $other) {
                let (res, overflow) = (*self).$overflowing_op(*other);
                $crate::panic_on_overflow!(overflow);
                *self = res;
            }       
        }   
    };
}



#[macro_export]
#[doc(hidden)]
macro_rules! impl_map_from {
	($thing:ident, $from:ty, $to:ty) => {
		impl From<$from> for $thing {
			fn from(value: $from) -> $thing {
				From::from(value as $to)
			}
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_try_from_for_primitive {
	($from:ident, $to:ty) => {
		impl std::convert::TryFrom<$from> for $to {
			type Error = &'static str;

			#[inline]
			fn try_from(u: $from) -> std::result::Result<$to, &'static str> {
				let $from(arr) = u;
				if !u.fits_word() || arr[0] > <$to>::max_value() as u64 {
					Err(concat!(
						"integer overflow when casting to ",
						stringify!($to)
					))
				} else {
					Ok(arr[0] as $to)
				}
			}
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_typecast_128 {
    ($name:ident, 1) => {
        impl From<u128> for $name {
            fn from(value: u128) -> $name {
                if value >= u64::max_value() as u128 {
                    panic!(concat!(
                        "integer overflow when casting to ",
                        stringify!($name)
                    ));
                }
                $name([value as u64])
            }
        }

        impl From<u128> for $name {
            fn from(value: u128) -> $name {
                if value < 0 || value >= u64::max_value() as i128 {
                    panic!(concat!(
                        "integer overflow when casting to ",
                        stringify!($name)
                    ));
                }
                $name([value as u64])
            }
        }

		impl std::convert::TryFrom<$name> for u128 {
			type Error = &'static str;

			#[inline]
			fn try_from(u: $name) -> std::result::Result<u128, &'static str> {
				let $name(arr) = u;
				Ok(arr[0] as u128)
			}
		}

		impl std::convert::TryFrom<$name> for i128 {
			type Error = &'static str;

			#[inline]
			fn try_from(u: $name) -> std::result::Result<i128, &'static str> {
				Ok(u128::from(u))
			}
		}

    };


    ($name:ident, $n_words:tt) => {
        impl From<u128> for $name {
            fn from(value: u128) -> $name {
                let mut ret = [0; $n_words];
                ret[0] = value as u64;
                ret[1] = (value >> 64) as u64;
                $name(ret)
            }
        }

        impl From<i128> for $name {
            fn from(value: i128) -> $name {
                if value < 0 {
                    panic!(concat!(
                        "integer overflow when casting to ",
                        stringify!($name)
                    ));
                }
                let mut ret = [0; $n_words];
                ret[0] = value as u64;
                ret[1] = (value >> 64) as u64;
                $name(ret)
            }
        }
        
        impl std::convert::TryFrom<$name> for u128 {
            type Error = &'static str;

            #[inline]
            fn try_from(u: $name) -> std::result::Result<u128, &'static str> {
                let $name(arr) = u;
                for i in 2..$n_words {
                    if arr[i] != 0 {
                        return Err("integer overflow when casting to u128");
                    }
                }
                Ok(((arr[1] as u128) << 64) + arr[0] as u128)
            }
        }

        impl std::convert::TryFrom<$name> for i128 {
            type Error = &'static str;

            #[inline]
            fn try_from(u: $name) -> std::result::Result<i128, &'static str> {
                let err_str = "integer overflow when casting to i128";
                let i = u128::try_from(u).map_err(|_| err_str)?;
                if i > i128::max_value() as u128 {
                    Err(err_str)
                } else {
                    Ok(i as i128)
                }
            }
        }

    };

}

