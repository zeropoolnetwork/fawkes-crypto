#![allow(unused_macros)]



#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_overflowing_binop{
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident for $name: ty, $method:ident, $overflowing_op: ident) => {
        impl_num_overflowing_binop!(impl <$($imp_l, )*$($imp_i : $imp_p),+> $op<$name> for $name, $method, $overflowing_op);
    };

	(impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident<$other:ty> for $name: ty, $method:ident, $overflowing_op: ident) => {
		impl <$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(other.0);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime $other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'macro_lifetime $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(other.0);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for &'macro_lifetime $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(other.0);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }

		impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime_a $other> for &'macro_lifetime_b $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'macro_lifetime_a $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(other.0);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_overflowing_binop_primitive{

	(impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident<$other:ty> for $name: ty, $method:ident, $overflowing_op: ident) => {
		impl <$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(other);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime $other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'macro_lifetime $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(*other);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for &'macro_lifetime $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(other);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }

		impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime_a $other> for &'macro_lifetime_b $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'macro_lifetime_a $other) -> Self::Output {
                let (res, overflow) = self.0.$overflowing_op(*other);
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_overflowing_unop {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident for $name: ty, $method:ident, $overflowing_op: ident) => {
        impl <$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op for $name {
            type Output = $name;
            #[inline]
            fn $method(self) -> $name {
                let (res, overflow) = self.0.$overflowing_op();
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }

        impl <'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op for &'macro_lifetime $name {
            type Output = $name;
            #[inline]
            fn $method(self) -> $name {
                let (res, overflow) = self.0.$overflowing_op();
                panic_on_overflow!(overflow);
                <$name>::new(res)
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_overflowing_assignop{
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident for $name: ty, $method:ident, $overflowing_op: ident) => {
        impl_num_overflowing_assignop!(impl <$($imp_l, )*$($imp_i : $imp_p),+> $op<$name> for $name, $method, $overflowing_op);
    };

	(impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident<$other:ty> for $name: ty, $method:ident, $overflowing_op: ident) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for $name {
            #[inline]
			fn $method(&mut self, other: $other) {
                let (res, overflow) = self.0.$overflowing_op(other.0);
                panic_on_overflow!(overflow);
                self.0 = res;
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime $other> for $name {
            #[inline]
			fn $method(&mut self, other: &'macro_lifetime $other) {
                let (res, overflow) = self.0.$overflowing_op(other.0);
                panic_on_overflow!(overflow);
                self.0 = res;
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_overflowing_assignop_primitive{
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident for $name: ty, $method:ident, $overflowing_op: ident) => {
        impl_num_overflowing_assignop!(impl <$($imp_l, )*$($imp_i : $imp_p),+> $op<$name> for $name, $method, $overflowing_op);
    };

	(impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident<$other:ty> for $name: ty, $method:ident, $overflowing_op: ident) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for $name {
            #[inline]
			fn $method(&mut self, other: $other) {
                let (res, overflow) = self.0.$overflowing_op(other);
                panic_on_overflow!(overflow);
                self.0 = res;
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime $other> for $name {
            #[inline]
			fn $method(&mut self, other: &'macro_lifetime $other) {
                let (res, overflow) = self.0.$overflowing_op(*other);
                panic_on_overflow!(overflow);
                self.0 = res;
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_map_from {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> From<$from:ty> for $to: ty) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> From<$from> for $to {
			fn from(value: $from) -> $to {
				<$to>::new(From::from(value))
			}
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_fnum_map_from {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> From<$from:ty> for $to: ty) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> From<$from> for $to {
			fn from(value: $from) -> $to {
				<$to>::from_uint_unchecked(From::from(value))
			}
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_fnum_map_from_signed {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> From<$from:ty> for $to: ty) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> From<$from> for $to {
			fn from(value: $from) -> $to {
                if value > 0 {
                    <$to>::from_uint_unchecked(From::from(value))
                } else {
                    -<$to>::from_uint_unchecked(From::from(-value))
                }
			}
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_try_from_for_primitive {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> TryFrom<$from:ty> for $to: ty) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> core::convert::TryFrom<$from> for $to {
			type Error = &'static str;

			#[inline]
			fn try_from(u: $from) -> core::result::Result<$to, &'static str> {
                match u.0.try_into() {
                    Ok(v)=>Ok(v),
                    _=> Err(concat!("integer overflow when casting to ", stringify!($to)))
                }

            }
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_fnum_try_from_for_primitive {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> TryFrom<$from:ty> for $to: ty) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> core::convert::TryFrom<$from> for $to {
			type Error = &'static str;

			#[inline]
			fn try_from(u: $from) -> core::result::Result<$to, &'static str> {
                match u.to_uint().try_into() {
                    Ok(v)=>Ok(v),
                    _=> Err(concat!("integer overflow when casting to ", stringify!($to)))
                }

            }
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_fnum_try_from_for_primitive_signed {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> TryFrom<$from:ty> for $to: ty) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> core::convert::TryFrom<$from> for $to {
			type Error = &'static str;

			#[inline]
			fn try_from(u: $from) -> core::result::Result<$to, &'static str> {
                let u = u.to_uint();
                match u.try_into() {
                    Ok(v)=>Ok(v),
                    _=> match (<$from>::MODULUS-u).try_into() {
                            Ok(v)=> {let v:$to = v; Ok(-v)}
                            _=> Err(concat!("integer overflow when casting to ", stringify!($to)))
                        }
                }

            }
		}
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_wrapping_binop{
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident for $name: ty, $method:ident, $wrapping_op: ident) => {
        impl_num_wrapping_binop!(impl <$($imp_l, )*$($imp_i : $imp_p),+> $op<$name> for $name, $method, $wrapping_op);
    };

	(impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident<$other:ty> for $name: ty, $method:ident, $wrapping_op: ident) => {
		impl <$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                <$name>::new(self.0.$wrapping_op(other.0))
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime $other> for $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'macro_lifetime $other) -> Self::Output {
                <$name>::new(self.0.$wrapping_op(other.0))
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for &'macro_lifetime $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: $other) -> Self::Output {
                <$name>::new(self.0.$wrapping_op(other.0))
            }
        }

		impl<'macro_lifetime_a, 'macro_lifetime_b, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime_a $other> for &'macro_lifetime_b $name {
			type Output = $name;

            #[inline]
			fn $method(self, other: &'macro_lifetime_a $other) -> Self::Output {
                <$name>::new(self.0.$wrapping_op(other.0))
            }
        }
	};
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_wrapping_unop {
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident for $name: ty, $method:ident, $wrapping_op: ident) => {
        impl <$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op for $name {
            type Output = $name;
            #[inline]
            fn $method(self) -> $name {
                <$name>::new(self.0.$wrapping_op())
            }
        }

        impl <'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op for &'macro_lifetime $name {
            type Output = $name;
            #[inline]
            fn $method(self) -> $name {
                <$name>::new(self.0.$wrapping_op())
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_num_wrapping_assignop{
    (impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident for $name: ty, $method:ident, $wrapping_op: ident) => {
        impl_num_wrapping_assignop!(impl <$($imp_l, )*$($imp_i : $imp_p),+> $op<$name> for $name, $method, $wrapping_op);
    };

	(impl <$($imp_l:lifetime, )*$($imp_i:ident : $imp_p:path),+> $op: ident<$other:ty> for $name: ty, $method:ident, $wrapping_op: ident) => {
		impl<$($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<$other> for $name {
            #[inline]
			fn $method(&mut self, other: $other) {
                self.0 = self.0.$wrapping_op(other.0);
            }
        }

		impl<'macro_lifetime, $($imp_l, )*$($imp_i : $imp_p),+> core::ops::$op<&'macro_lifetime $other> for $name {
            #[inline]
			fn $method(&mut self, other: &'macro_lifetime $other) {
                self.0 = self.0.$wrapping_op(other.0);
            }
        }
    };
}
