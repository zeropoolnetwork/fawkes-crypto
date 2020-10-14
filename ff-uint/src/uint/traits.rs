pub trait Uint:
    Sized
    + Clone
    + Copy
    + Default
    + std::cmp::PartialEq
    + std::cmp::Eq
    + std::cmp::PartialOrd
    + std::cmp::Ord
    + From<bool>
    + From<u8>
    + From<u16>
    + From<u32>
    + From<u64>
    + From<u128>
    + From<i8>
    + From<i16>
    + From<i32>
    + From<i64>
    + From<i128>
    + std::convert::TryInto<bool>
    + std::convert::TryInto<u8>
    + std::convert::TryInto<u16>
    + std::convert::TryInto<u32>
    + std::convert::TryInto<u64>
    + std::convert::TryInto<u128>
    + std::convert::TryInto<i8>
    + std::convert::TryInto<i16>
    + std::convert::TryInto<i32>
    + std::convert::TryInto<i64>
    + std::convert::TryInto<i128>
    + std::hash::Hash
    + std::fmt::Debug
    + std::fmt::Display
    + std::str::FromStr
    + std::fmt::LowerHex
    + From<&'static str>
    + crate::borsh::BorshSerialize
    + crate::borsh::BorshDeserialize
{
    type Inner: AsMut<[u64]> + AsRef<[u64]> + Copy + Clone + Default + Sized;

    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;

    const NUM_WORDS: usize;
    const WORD_BITS: usize;

    fn max_value() -> Self {
        Self::MAX
    }
    fn min_value() -> Self {
        Self::ZERO
    }

    fn is_even(&self) -> bool {
        !self.bit(0)
    }

    fn is_odd(&self) -> bool {
        self.bit(0)
    }

    fn random<R: rand::Rng + ?Sized>(rng: &mut R) -> Self;

    fn into_inner(self) -> Self::Inner;
    fn as_inner(&self) -> &Self::Inner;
    fn as_inner_mut(&mut self) -> &mut Self::Inner;

    fn put_big_endian(&self, bytes: &mut [u8]);
    fn put_little_endian(&self, bytes: &mut [u8]);
    fn to_big_endian(&self) -> Vec<u8>;
    fn to_little_endian(&self) -> Vec<u8>;
    fn from_big_endian(slice: &[u8]) -> Self;
    fn from_little_endian(slice: &[u8]) -> Self;

    fn as_u64(&self) -> u64;
    fn low_u64(&self) -> u64;
    fn from_u64(v: u64) -> Self;

    fn is_zero(&self) -> bool;
    fn bits(&self) -> usize;

    fn bit(&self, n: usize) -> bool {
        if n >= Self::NUM_WORDS * Self::WORD_BITS {
            panic!("Bit index overflow")
        } else {
            let limb = n / Self::WORD_BITS;
            let bitpos = n % Self::WORD_BITS;
            (self.as_inner().as_ref()[limb] >> bitpos) & 1 == 1
        }
    }
    fn leading_zeros(&self) -> u32;
    fn trailing_zeros(&self) -> u32;

    fn div_mod(self, other: Self) -> (Self, Self);

    fn overflowing_add(self, other: Self) -> (Self, bool);
    fn overflowing_sub(self, other: Self) -> (Self, bool);
    fn overflowing_mul_u64(self, other: u64) -> (Self, u64);
    fn overflowing_mul(self, other: Self) -> (Self, bool);

    fn overflowing_not(self) -> (Self, bool);
    fn overflowing_bitand(self, other: Self) -> (Self, bool);
    fn overflowing_bitor(self, other: Self) -> (Self, bool);
    fn overflowing_bitxor(self, other: Self) -> (Self, bool);

    fn overflowing_neg(self) -> (Self, bool);
    fn overflowing_shr(self, other: u32) -> (Self, bool);
    fn overflowing_shl(self, other: u32) -> (Self, bool);

    #[inline]
    fn overflowing_pow<S: BitIterBE>(self, exp: S) -> (Self, bool) {
        let mut res = Self::ONE;
        let mut overflow: bool = false;
        let mut found_one = false;
        for i in exp.bit_iter_be() {
            if found_one {
                res = overflowing!(res.overflowing_mul(res), overflow);
            } else {
                found_one = i;
            }
            if i {
                res = overflowing!(res.overflowing_mul(self), overflow);
            }
        }
        (res, overflow)
    }

    #[inline]
    fn to_other<U: Uint>(self) -> Option<U> {
        let mut res = U::default();
        let res_inner = res.as_inner_mut().as_mut();
        let res_inner_len = res_inner.len();

        let self_inner = self.as_inner().as_ref();
        let self_inner_len = self_inner.len();

        let both_min = std::cmp::min(res_inner_len, self_inner_len);

        res_inner[..both_min].copy_from_slice(&self_inner[..both_min]);

        if self_inner[both_min..].iter().any(|&x| x != 0) {
            None
        } else {
            Some(res)
        }
    }

    fn wrapping_cmp(&self, other: &Self) -> std::cmp::Ordering;

    #[inline]
    fn unchecked_pow(self, other: Self) -> Self {
        self.overflowing_pow(other).0
    }

    #[inline]
    fn unchecked_add(self, other: Self) -> Self {
        self.overflowing_add(other).0
    }

    #[inline]
    fn unchecked_sub(self, other: Self) -> Self {
        self.overflowing_sub(other).0
    }

    #[inline]
    fn unchecked_mul(self, other: Self) -> Self {
        self.overflowing_mul(other).0
    }

    /// Checked division. Returns `None` if `other == 0`.
    #[inline]
    fn overflowing_div(self, other: Self) -> (Self, bool) {
        (self.div_mod(other).0, false)
    }

    #[inline]
    fn unchecked_div(self, other: Self) -> Self {
        self.div_mod(other).0
    }

    #[inline]
    fn overflowing_rem(self, other: Self) -> (Self, bool) {
        (self.div_mod(other).1, false)
    }

    #[inline]
    fn unchecked_rem(self, other: Self) -> Self {
        self.div_mod(other).1
    }

    #[inline]
    fn unchecked_neg(self) -> Self {
        self.overflowing_neg().0
    }

    #[inline]
    fn unchecked_shr(self, rhs: u32) -> Self {
        self.overflowing_shr(rhs).0
    }

    #[inline]
    fn unchecked_shl(self, lhs: u32) -> Self {
        self.overflowing_shl(lhs).0
    }

    crate::impl_wrapping_bin_method!(wrapping_pow, overflowing_pow, Self);
    crate::impl_wrapping_bin_method!(wrapping_add, overflowing_add, Self);
    crate::impl_wrapping_bin_method!(wrapping_sub, overflowing_sub, Self);
    crate::impl_wrapping_bin_method!(wrapping_mul, overflowing_mul, Self);
    crate::impl_wrapping_bin_method!(wrapping_div, overflowing_div, Self);
    crate::impl_wrapping_bin_method!(wrapping_rem, overflowing_rem, Self);
    crate::impl_wrapping_bin_method!(wrapping_shl, overflowing_shl, u32);
    crate::impl_wrapping_bin_method!(wrapping_shr, overflowing_shr, u32);
    crate::impl_wrapping_un_method!(wrapping_neg, overflowing_neg);
    crate::impl_wrapping_un_method!(wrapping_not, overflowing_not);
}

pub struct BitIteratorLE<E> {
    t: E,
    i: usize,
    n: usize,
}

impl<E: Uint> Iterator for BitIteratorLE<E> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.i >= self.n {
            None
        } else {
            let part = self.i / 64;
            let bit = self.i & 63;
            self.i += 1;
            Some((self.t.as_inner().as_ref()[part] >> bit) & 1 == 1)
        }
    }
}

pub trait BitIterLE {
    type Iter: Iterator<Item = bool>;

    fn bit_iter_le(&self) -> Self::Iter;
}

impl<I: Uint> BitIterLE for I {
    type Iter = BitIteratorLE<I>;

    fn bit_iter_le(&self) -> Self::Iter {
        Self::Iter {
            t: *self,
            i: 0,
            n: I::NUM_WORDS * I::WORD_BITS,
        }
    }
}

pub struct BitIteratorBE<E> {
    t: E,
    i: usize,
}

impl<E: Uint> Iterator for BitIteratorBE<E> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.i == 0 {
            None
        } else {
            self.i -= 1;
            let part = self.i / 64;
            let bit = self.i & 63;
            Some((self.t.as_inner().as_ref()[part] >> bit) & 1 == 1)
        }
    }
}

pub trait BitIterBE {
    type Iter: Iterator<Item = bool>;

    fn bit_iter_be(&self) -> Self::Iter;
}

impl<I: Uint> BitIterBE for I {
    type Iter = BitIteratorBE<I>;

    fn bit_iter_be(&self) -> Self::Iter {
        Self::Iter {
            t: *self,
            i: I::NUM_WORDS * I::WORD_BITS,
        }
    }
}
