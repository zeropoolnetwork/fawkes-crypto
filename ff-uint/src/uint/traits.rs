pub trait Uint:
    Sized
    + Clone
    + Copy
    + Default
    + std::cmp::PartialEq
    + std::cmp::Eq
    + std::cmp::PartialOrd
    + std::cmp::Ord
    + std::ops::Add<Self, Output = Self>
    + std::ops::Sub<Self, Output = Self>
    + std::ops::Mul<Self, Output = Self>
    + std::ops::Mul<u64, Output = Self>
    + std::ops::Div<Self, Output = Self>
    + std::ops::Rem<Self, Output = Self>
    + std::ops::Shl<u32, Output = Self>
    + std::ops::Shr<u32, Output = Self>
    + std::ops::AddAssign<Self>
    + std::ops::SubAssign<Self>
    + std::ops::MulAssign<Self>
    + std::ops::MulAssign<u64>
    + std::ops::DivAssign<Self>
    + std::ops::RemAssign<Self>
    + std::ops::Not<Output = Self>
    + std::ops::BitAnd<Self, Output = Self>
    + std::ops::BitOr<Self, Output = Self>
    + std::ops::BitXor<Self, Output = Self>
    + std::ops::BitAndAssign<Self>
    + std::ops::BitOrAssign<Self>
    + std::ops::ShlAssign<u32>
    + std::ops::ShrAssign<u32>
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
    fn wrapping_pow(self, other: Self) -> Self {
        self.overflowing_pow(other).0
    }

    #[inline]
    fn checked_pow(self, expon: Self) -> Option<Self> {
        match self.overflowing_pow(expon) {
            (_, true) => None,
            (val, _) => Some(val),
        }
    }

    #[inline]
    fn saturating_pow(self, other: Self) -> Self {
        match self.overflowing_pow(other) {
            (_, true) => Self::MAX,
            (val, false) => val,
        }
    }

    #[inline]
    fn wrapping_add(self, other: Self) -> Self {
        self.overflowing_add(other).0
    }

    #[inline]
    fn saturating_add(self, other: Self) -> Self {
        match self.overflowing_add(other) {
            (_, true) => Self::MAX,
            (val, false) => val,
        }
    }

    #[inline]
    fn checked_add(self, other: Self) -> Option<Self> {
        match self.overflowing_add(other) {
            (_, true) => None,
            (val, _) => Some(val),
        }
    }

    #[inline]
    fn wrapping_sub(self, other: Self) -> Self {
        self.overflowing_sub(other).0
    }

    #[inline]
    fn saturating_sub(self, other: Self) -> Self {
        match self.overflowing_sub(other) {
            (_, true) => Self::ZERO,
            (val, false) => val,
        }
    }

    #[inline]
    fn checked_sub(self, other: Self) -> Option<Self> {
        match self.overflowing_sub(other) {
            (_, true) => None,
            (val, _) => Some(val),
        }
    }

    #[inline]
    fn wrapping_mul(self, other: Self) -> Self {
        self.overflowing_mul(other).0
    }

    #[inline]
    fn saturating_mul(self, other: Self) -> Self {
        match self.overflowing_mul(other) {
            (_, true) => Self::MAX,
            (val, false) => val,
        }
    }

    #[inline]
    fn checked_mul(self, other: Self) -> Option<Self> {
        match self.overflowing_mul(other) {
            (_, true) => None,
            (val, _) => Some(val),
        }
    }

    #[inline]
    fn checked_div(self, other: Self) -> Option<Self> {
        if other.is_zero() {
            None
        } else {
            Some(self.div_mod(other).0)
        }
    }

    /// Checked division. Returns `None` if `other == 0`.
    #[inline]
    fn overflowing_div(self, other: Self) -> (Self, bool) {
        (self.div_mod(other).0, false)
    }

    #[inline]
    fn wrapping_div(self, other: Self) -> Self {
        self.div_mod(other).0
    }

    /// Checked modulus. Returns `None` if `other == 0`.
    #[inline]
    fn checked_rem(self, other: Self) -> Option<Self> {
        if other.is_zero() {
            None
        } else {
            Some(self.div_mod(other).1)
        }
    }

    #[inline]
    fn overflowing_rem(self, other: Self) -> (Self, bool) {
        (self.div_mod(other).1, false)
    }

    #[inline]
    fn wrapping_rem(self, other: Self) -> Self {
        self.div_mod(other).1
    }

    #[inline]
    fn wrapping_neg(self) -> Self {
        self.overflowing_neg().0
    }

    /// Checked negation. Returns `None` unless `self == 0`.
    #[inline]
    fn checked_neg(self) -> Option<Self> {
        match self.overflowing_neg() {
            (_, true) => None,
            (zero, false) => Some(zero),
        }
    }

    #[inline]
    fn wrapping_shr(self, rhs: u32) -> Self {
        self.overflowing_shr(rhs).0
    }

    #[inline]
    fn checked_shr(self, rhs: u32) -> Option<Self> {
        match self.overflowing_shr(rhs) {
            (_, true) => None,
            (val, false) => Some(val),
        }
    }

    #[inline]
    fn wrapping_shl(self, lhs: u32) -> Self {
        self.overflowing_shl(lhs).0
    }

    #[inline]
    fn checked_shl(self, lhs: u32) -> Option<Self> {
        match self.overflowing_shl(lhs) {
            (_, true) => None,
            (val, false) => Some(val),
        }
    }
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
