#[macro_use]
pub(crate) mod macros;

#[cfg(feature = "borsh_support")]
use crate::borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::{PrimeField, Uint};
#[cfg(not(feature = "std"))]
use alloc::vec;

use ref_cast::RefCast;

use crate::seedbox::{SeedBox, SeedBoxGen, SeedboxBlake2};
use core::convert::TryInto;

#[repr(transparent)]
#[derive(Clone, Copy, RefCast)]
pub struct NumRepr<U: Uint>(pub U);

#[repr(transparent)]
#[derive(Clone, Copy, RefCast)]
pub struct Num<Fp: PrimeField>(pub Fp);

// num ops for Uint

impl<U: Uint> NumRepr<U> {
    pub const ONE: Self = NumRepr(U::ONE);
    pub const ZERO: Self = NumRepr(U::ZERO);
    pub const MAX: Self = NumRepr(U::MAX);

    pub fn new(n: U) -> Self {
        Self(n)
    }

    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    pub fn is_even(&self) -> bool {
        self.0.is_even()
    }

    pub fn is_odd(&self) -> bool {
        self.0.is_odd()
    }

    pub fn into_inner(self) -> U::Inner {
        self.0.into_inner()
    }
    pub fn as_inner(&self) -> &U::Inner {
        self.0.as_inner()
    }
    pub fn as_inner_mut(&mut self) -> &mut U::Inner {
        self.0.as_inner_mut()
    }
}

#[cfg(feature = "rand_support")]
impl<U: Uint> crate::rand::distributions::Distribution<NumRepr<U>>
    for crate::rand::distributions::Standard
{
    #[inline]
    fn sample<R: crate::rand::Rng + ?Sized>(&self, rng: &mut R) -> NumRepr<U> {
        NumRepr::new(U::random(rng))
    }
}

#[cfg(feature = "borsh_support")]
impl<U: Uint> BorshSerialize for NumRepr<U> {
    fn serialize<W: borsh::lib::Write>(&self, writer: &mut W) -> Result<(), borsh::error::Error> {
        self.0.serialize(writer)
    }
}

#[cfg(feature = "borsh_support")]
impl<U: Uint> BorshDeserialize for NumRepr<U> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, borsh::error::Error> {
        Ok(Self(U::deserialize(buf)?))
    }
}

#[cfg(feature = "borsh_support")]
impl<U: Uint> Serialize for NumRepr<U> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.to_string(), serializer)
    }
}

#[cfg(feature = "borsh_support")]
impl<'de, U: Uint> Deserialize<'de> for NumRepr<U> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        core::str::FromStr::from_str(&<String as Deserialize>::deserialize(deserializer)?)
            .map_err(|_| crate::serde::de::Error::custom("Wrong number format"))
    }
}

impl_num_overflowing_unop!(impl <U:Uint> Not for NumRepr<U>, not, overflowing_not);
impl_num_overflowing_unop!(impl <U:Uint> Neg for NumRepr<U>, neg, overflowing_neg);

impl_num_overflowing_binop!(impl <U:Uint> Add for NumRepr<U>, add, overflowing_add);
impl_num_overflowing_binop!(impl <U:Uint> Sub for NumRepr<U>, sub, overflowing_sub);
impl_num_overflowing_binop!(impl <U:Uint> Mul for NumRepr<U>, mul, overflowing_mul);
impl_num_overflowing_binop_primitive!(impl <U:Uint> Mul<u64> for NumRepr<U>, mul, overflowing_mul_u64);
impl_num_overflowing_binop!(impl <U:Uint> Div for NumRepr<U>, div, overflowing_div);
impl_num_overflowing_binop!(impl <U:Uint> Rem for NumRepr<U>, rem, overflowing_rem);
impl_num_overflowing_binop_primitive!(impl <U:Uint> Shr<u32> for NumRepr<U>, shr, overflowing_shr);
impl_num_overflowing_binop_primitive!(impl <U:Uint> Shl<u32> for NumRepr<U>, shl, overflowing_shl);

impl_num_overflowing_binop!(impl <U:Uint> BitAnd for NumRepr<U>, bitand, overflowing_bitand);
impl_num_overflowing_binop!(impl <U:Uint> BitOr for NumRepr<U>, bitor, overflowing_bitor);
impl_num_overflowing_binop!(impl <U:Uint> BitXor for NumRepr<U>, bitxor, overflowing_bitxor);

impl_num_overflowing_assignop!(impl <U:Uint> AddAssign for NumRepr<U>, add_assign, overflowing_add);
impl_num_overflowing_assignop!(impl <U:Uint> SubAssign for NumRepr<U>, sub_assign, overflowing_sub);
impl_num_overflowing_assignop!(impl <U:Uint> MulAssign for NumRepr<U>, mul_assign, overflowing_mul);
impl_num_overflowing_assignop_primitive!(impl <U:Uint> MulAssign<u64> for NumRepr<U>, mul_assign, overflowing_mul_u64);
impl_num_overflowing_assignop!(impl <U:Uint> DivAssign for NumRepr<U>, div_assign, overflowing_div);
impl_num_overflowing_assignop!(impl <U:Uint> RemAssign for NumRepr<U>, rem_assign, overflowing_rem);
impl_num_overflowing_assignop_primitive!(impl <U:Uint> ShrAssign<u32> for NumRepr<U>, shr_assign, overflowing_shr);
impl_num_overflowing_assignop_primitive!(impl <U:Uint> ShlAssign<u32> for NumRepr<U>, shl_assign, overflowing_shl);

impl_num_overflowing_assignop!(impl <U:Uint> BitAndAssign for NumRepr<U>, bitand_assign, overflowing_bitand);
impl_num_overflowing_assignop!(impl <U:Uint> BitOrAssign for NumRepr<U>, bitor_assign, overflowing_bitor);
impl_num_overflowing_assignop!(impl <U:Uint> BitXorAssign for NumRepr<U>, bitxor_assign, overflowing_bitxor);

impl_num_map_from!(impl<U:Uint> From<bool> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<u8> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<u16> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<u32> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<u64> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<u128> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<i8> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<i16> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<i32> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<i64> for NumRepr<U>);
impl_num_map_from!(impl<U:Uint> From<i128> for NumRepr<U>);

impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for bool);

impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u8);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u16);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u32);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u64);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u128);

impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i8);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i16);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i32);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i64);
impl_num_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i128);

impl<U: Uint> core::cmp::Ord for NumRepr<U> {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.wrapping_cmp(&other.0)
    }
}

impl<U: Uint> core::cmp::PartialOrd for NumRepr<U> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<U: Uint> core::cmp::PartialEq for NumRepr<U> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<U: Uint> core::cmp::Eq for NumRepr<U> {}

impl<U: Uint> core::hash::Hash for NumRepr<U> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<U: Uint> core::fmt::Debug for NumRepr<U> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self, f)
    }
}

impl<U: Uint> core::fmt::Display for NumRepr<U> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.is_zero() {
            return core::write!(f, "0");
        }

        // error: constant expression depends on a generic parameter
        // let mut buf = [0_u8; U::NUM_WORDS * 20];

        let mut buf = vec![0u8; U::NUM_WORDS * 20];
        let mut i = buf.len() - 1;
        let mut current = self.0;
        let ten = U::from_u64(10);
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
        let s = unsafe { core::str::from_utf8_unchecked(&buf[i..]) };
        f.write_str(s)
    }
}

impl<U: Uint> core::fmt::LowerHex for NumRepr<U> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if f.alternate() {
            core::write!(f, "0x")?;
        }
        // special case.
        if self.is_zero() {
            return core::write!(f, "0");
        }

        let mut latch = false;
        for ch in self.0.as_inner().as_ref().iter().rev() {
            for x in 0..16 {
                let nibble = (ch & (15u64 << ((15 - x) * 4) as u64)) >> (((15 - x) * 4) as u64);
                if !latch {
                    latch = nibble != 0;
                }

                if latch {
                    core::write!(f, "{:x}", nibble)?;
                }
            }
        }
        Ok(())
    }
}

impl<U: Uint> core::str::FromStr for NumRepr<U> {
    type Err = <U as core::str::FromStr>::Err;

    fn from_str(value: &str) -> core::result::Result<NumRepr<U>, Self::Err> {
        Ok(<NumRepr<U>>::new(U::from_str(value)?))
    }
}

impl<U: Uint> core::convert::From<&'static str> for NumRepr<U> {
    fn from(s: &'static str) -> Self {
        <NumRepr<U>>::new(U::from(s))
    }
}

// num ops for PrimeField
#[cfg(feature = "rand_support")]
impl<Fp: PrimeField> crate::rand::distributions::Distribution<Num<Fp>>
    for crate::rand::distributions::Standard
{
    #[inline]
    fn sample<R: crate::rand::Rng + ?Sized>(&self, rng: &mut R) -> Num<Fp> {
        Num::new(Fp::random(rng))
    }
}

#[cfg(feature = "borsh_support")]
impl<Fp: PrimeField> BorshSerialize for Num<Fp> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        self.0.serialize(writer)
    }
}

#[cfg(feature = "borsh_support")]
impl<Fp: PrimeField> BorshDeserialize for Num<Fp> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self(Fp::deserialize(buf)?))
    }
}

impl<Fp: PrimeField> SeedBoxGen<Num<Fp>> for SeedboxBlake2 {
    fn gen(&mut self) -> Num<Fp> {
        let mut n = Fp::Inner::ZERO;
        let shave = 0xffffffffffffffffu64 >> Fp::REPR_SHAVE_BITS;
        let len = Fp::Inner::NUM_WORDS;
        loop {
            {
                let p = n.as_inner_mut().as_mut();
                self.fill_limbs(p);
                p[len-1] &= shave;
            }
            match Num::from_mont_uint(NumRepr(n)) {
                Some(n) => return n,
                _ => {}
            }
        }
    }
}

impl<Fp: PrimeField> Num<Fp> {
    pub const ZERO: Self = Num(Fp::ZERO);
    pub const ONE: Self = Num(Fp::ONE);
    pub const MODULUS: NumRepr<Fp::Inner> = NumRepr(Fp::MODULUS);
    pub const MODULUS_BITS: u32 = Fp::MODULUS_BITS;

    pub fn is_even(&self) -> bool {
        self.to_uint().0.is_even()
    }

    pub fn is_odd(&self) -> bool {
        self.to_uint().0.is_odd()
    }

    pub fn double(self) -> Self {
        Self(self.0.double())
    }

    pub fn square(self) -> Self {
        Self(self.0.square())
    }

    pub fn new(n: Fp) -> Self {
        Self(n)
    }

    pub fn checked_inv(self) -> Option<Self> {
        Some(Self(self.0.checked_inv()?))
    }

    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    pub fn from_uint(v: NumRepr<Fp::Inner>) -> Option<Self> {
        Some(Self(Fp::from_uint(v.0)?))
    }

    pub fn from_uint_reduced(v: NumRepr<Fp::Inner>) -> Self {
        let n = v % Num::<Fp>::MODULUS;
        Self(Fp::from_uint_unchecked(n.0))
    }

    pub fn from_mont_uint(v: NumRepr<Fp::Inner>) -> Option<Self> {
        Some(Self(Fp::from_mont_uint(v.0)?))
    }

    pub fn from_uint_unchecked(v: NumRepr<Fp::Inner>) -> Self {
        Self(Fp::from_uint_unchecked(v.0))
    }

    pub fn from_mont_uint_unchecked(v: NumRepr<Fp::Inner>) -> Self {
        Self(Fp::from_mont_uint_unchecked(v.0))
    }

    pub fn sqrt(&self) -> Option<Self> {
        Some(Self(self.0.sqrt()?))
    }

    pub fn even_sqrt(&self) -> Option<Self> {
        let res = self.sqrt()?;
        if res.to_uint().is_even() {
            Some(res)
        } else {
            Some(-res)
        }
    }

    pub fn to_uint(&self) -> NumRepr<Fp::Inner> {
        NumRepr(self.0.to_uint())
    }

    pub fn to_mont_uint(&self) -> NumRepr<Fp::Inner> {
        NumRepr(self.0.to_mont_uint())
    }

    pub fn as_mont_uint(&self) -> &NumRepr<Fp::Inner> {
        NumRepr::ref_cast(self.0.as_mont_uint())
    }

    pub fn as_mont_uint_mut(&mut self) -> &mut NumRepr<Fp::Inner> {
        NumRepr::ref_cast_mut(self.0.as_mont_uint_mut())
    }

    pub fn to_other<Fq: PrimeField>(&self) -> Option<Num<Fq>> {
        Some(Num(self.0.to_other()?))
    }

    pub fn to_other_reduced<Fq: PrimeField>(&self) -> Num<Fq> {
        Num(self.0.to_other_reduced())
    }
}

impl_num_wrapping_unop!(impl <U:PrimeField> Neg for Num<U>, neg, wrapping_neg);
impl_num_wrapping_binop!(impl <U:PrimeField> Add for Num<U>, add, wrapping_add);
impl_num_wrapping_binop!(impl <U:PrimeField> Sub for Num<U>, sub, wrapping_sub);
impl_num_wrapping_binop!(impl <U:PrimeField> Mul for Num<U>, mul, wrapping_mul);
impl_num_wrapping_binop!(impl <U:PrimeField> Div for Num<U>, div, wrapping_div);

impl_num_wrapping_assignop!(impl <U:PrimeField> AddAssign for Num<U>, add_assign, wrapping_add);
impl_num_wrapping_assignop!(impl <U:PrimeField> SubAssign for Num<U>, sub_assign, wrapping_sub);
impl_num_wrapping_assignop!(impl <U:PrimeField> MulAssign for Num<U>, mul_assign, wrapping_mul);
impl_num_wrapping_assignop!(impl <U:PrimeField> DivAssign for Num<U>, div_assign, wrapping_div);

impl<Fp: PrimeField> core::cmp::PartialEq for Num<Fp> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<Fp: PrimeField> core::cmp::Eq for Num<Fp> {}

impl<Fp: PrimeField> core::fmt::Debug for Num<Fp> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.0, f)
    }
}

impl<Fp: PrimeField> core::fmt::Display for Num<Fp> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

impl<Fp: PrimeField> core::str::FromStr for Num<Fp> {
    type Err = <Fp as core::str::FromStr>::Err;

    fn from_str(value: &str) -> core::result::Result<Num<Fp>, Self::Err> {
        Ok(<Num<Fp>>::new(Fp::from_str(value)?))
    }
}

impl<Fp: PrimeField> core::convert::From<&'static str> for Num<Fp> {
    fn from(s: &'static str) -> Self {
        <Num<Fp>>::new(Fp::from(s))
    }
}

#[cfg(feature = "serde_support")]
impl<Fp: PrimeField> Serialize for Num<Fp> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.to_string(), serializer)
    }
}

#[cfg(feature = "serde_support")]
impl<'de, Fp: PrimeField> Deserialize<'de> for Num<Fp> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bn = <NumRepr<Fp::Inner> as Deserialize>::deserialize(deserializer)?;
        Self::from_uint(bn).ok_or(crate::serde::de::Error::custom("Field overflow"))
    }
}

impl<Fp: PrimeField> From<bool> for Num<Fp> {
    fn from(n: bool) -> Self {
        match n {
            false => Self::ZERO,
            true => Self::ONE,
        }
    }
}

impl_fnum_map_from!(impl<Fp:PrimeField> From<u8> for Num<Fp>);
impl_fnum_map_from!(impl<Fp:PrimeField> From<u16> for Num<Fp>);
impl_fnum_map_from!(impl<Fp:PrimeField> From<u32> for Num<Fp>);
impl_fnum_map_from!(impl<Fp:PrimeField> From<u64> for Num<Fp>);
impl_fnum_map_from!(impl<Fp:PrimeField> From<u128> for Num<Fp>);

impl_fnum_map_from_signed!(impl<Fp:PrimeField> From<i8> for Num<Fp>);
impl_fnum_map_from_signed!(impl<Fp:PrimeField> From<i16> for Num<Fp>);
impl_fnum_map_from_signed!(impl<Fp:PrimeField> From<i32> for Num<Fp>);
impl_fnum_map_from_signed!(impl<Fp:PrimeField> From<i64> for Num<Fp>);
impl_fnum_map_from_signed!(impl<Fp:PrimeField> From<i128> for Num<Fp>);

impl<Fp: PrimeField> core::convert::TryFrom<Num<Fp>> for bool {
    type Error = &'static str;

    #[inline]
    fn try_from(u: Num<Fp>) -> core::result::Result<bool, &'static str> {
        match u.to_uint().try_into() {
            Ok(v) => Ok(v),
            _ => Err(concat!("integer overflow when casting to bool")),
        }
    }
}

impl_fnum_try_from_for_primitive!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for u8);
impl_fnum_try_from_for_primitive!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for u16);
impl_fnum_try_from_for_primitive!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for u32);
impl_fnum_try_from_for_primitive!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for u64);
impl_fnum_try_from_for_primitive!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for u128);

impl_fnum_try_from_for_primitive_signed!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for i8);
impl_fnum_try_from_for_primitive_signed!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for i16);
impl_fnum_try_from_for_primitive_signed!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for i32);
impl_fnum_try_from_for_primitive_signed!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for i64);
impl_fnum_try_from_for_primitive_signed!(impl<Fp:PrimeField> TryFrom<Num<Fp>> for i128);

use crate::{BitIterBE, BitIterLE, BitIteratorBE, BitIteratorLE};

impl<I: Uint> BitIterBE for NumRepr<I> {
    type Iter = BitIteratorBE<I>;

    fn bit_iter_be(&self) -> Self::Iter {
        self.0.bit_iter_be()
    }
}

impl<I: Uint> BitIterLE for NumRepr<I> {
    type Iter = BitIteratorLE<I>;

    fn bit_iter_le(&self) -> Self::Iter {
        self.0.bit_iter_le()
    }
}

impl<Fp: PrimeField> BitIterBE for Num<Fp> {
    type Iter = BitIteratorBE<Fp::Inner>;

    fn bit_iter_be(&self) -> Self::Iter {
        self.to_uint().bit_iter_be()
    }
}

impl<Fp: PrimeField> BitIterLE for Num<Fp> {
    type Iter = BitIteratorLE<Fp::Inner>;

    fn bit_iter_le(&self) -> Self::Iter {
        self.to_uint().bit_iter_le()
    }
}
