
#[macro_use]
pub(crate) mod macros;

use crate::{PrimeField, Uint};
use crate::borsh::{BorshDeserialize, BorshSerialize};
use ref_cast::RefCast;
use crate::serde::{Serialize, Serializer, Deserialize, Deserializer};

#[repr(transparent)]
#[derive(Clone, Copy, RefCast)]
pub struct NumRepr<U:Uint>(pub U);

#[repr(transparent)]
#[derive(Clone, Copy, RefCast)]
pub struct Num<Fp:PrimeField>(pub Fp);


// Wrapped ops for Uint

impl <U:Uint> NumRepr<U> {
    pub const ONE: Self = NumRepr(U::ONE);
    pub const ZERO: Self = NumRepr(U::ZERO); 
    pub const MAX: Self = NumRepr(U::MAX);

    pub fn new(n:U) -> Self {
        Self(n)
    }
}

impl<U:Uint> BorshSerialize for NumRepr<U> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        self.0.serialize(writer)
    }
}

impl<U:Uint> BorshDeserialize for NumRepr<U> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self(U::deserialize(buf)?))
    }
}


impl<U:Uint> Serialize for NumRepr<U> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>{
        Serialize::serialize(&self.to_string(), serializer)
    }
}

impl<'de, U:Uint> Deserialize<'de> for NumRepr<U> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        std::str::FromStr::from_str(&<String as Deserialize>::deserialize(deserializer)?).map_err(|_| crate::serde::de::Error::custom("Wrong number format"))
    }
}


impl_wrapped_overflowing_unop!(impl <U:Uint> Not for NumRepr<U>, not, overflowing_not);
impl_wrapped_overflowing_unop!(impl <U:Uint> Neg for NumRepr<U>, neg, overflowing_neg);


impl_wrapped_overflowing_binop!(impl <U:Uint> Add for NumRepr<U>, add, overflowing_add);
impl_wrapped_overflowing_binop!(impl <U:Uint> Sub for NumRepr<U>, sub, overflowing_sub);
impl_wrapped_overflowing_binop!(impl <U:Uint> Mul for NumRepr<U>, mul, overflowing_mul);
impl_wrapped_overflowing_binop_primitive!(impl <U:Uint> Mul<u64> for NumRepr<U>, mul, overflowing_mul_u64);
impl_wrapped_overflowing_binop!(impl <U:Uint> Div for NumRepr<U>, div, overflowing_div);
impl_wrapped_overflowing_binop!(impl <U:Uint> Rem for NumRepr<U>, rem, overflowing_rem);
impl_wrapped_overflowing_binop_primitive!(impl <U:Uint> Shr<u32> for NumRepr<U>, shr, overflowing_shr);
impl_wrapped_overflowing_binop_primitive!(impl <U:Uint> Shl<u32> for NumRepr<U>, shl, overflowing_shl);

impl_wrapped_overflowing_binop!(impl <U:Uint> BitAnd for NumRepr<U>, bitand, overflowing_bitand);
impl_wrapped_overflowing_binop!(impl <U:Uint> BitOr for NumRepr<U>, bitor, overflowing_bitor);
impl_wrapped_overflowing_binop!(impl <U:Uint> BitXor for NumRepr<U>, bitxor, overflowing_bitxor);


impl_wrapped_overflowing_assignop!(impl <U:Uint> AddAssign for NumRepr<U>, add_assign, overflowing_add);
impl_wrapped_overflowing_assignop!(impl <U:Uint> SubAssign for NumRepr<U>, sub_assign, overflowing_sub);
impl_wrapped_overflowing_assignop!(impl <U:Uint> MulAssign for NumRepr<U>, mul_assign, overflowing_mul);
impl_wrapped_overflowing_assignop_primitive!(impl <U:Uint> MulAssign<u64> for NumRepr<U>, mul_assign, overflowing_mul_u64);
impl_wrapped_overflowing_assignop!(impl <U:Uint> DivAssign for NumRepr<U>, div_assign, overflowing_div);
impl_wrapped_overflowing_assignop!(impl <U:Uint> RemAssign for NumRepr<U>, rem_assign, overflowing_rem);
impl_wrapped_overflowing_assignop_primitive!(impl <U:Uint> ShrAssign<u32> for NumRepr<U>, shr_assign, overflowing_shr);
impl_wrapped_overflowing_assignop_primitive!(impl <U:Uint> ShlAssign<u32> for NumRepr<U>, shl_assign, overflowing_shl);

impl_wrapped_overflowing_assignop!(impl <U:Uint> BitAndAssign for NumRepr<U>, bitand_assign, overflowing_bitand);
impl_wrapped_overflowing_assignop!(impl <U:Uint> BitOrAssign for NumRepr<U>, bitor_assign, overflowing_bitor);
impl_wrapped_overflowing_assignop!(impl <U:Uint> BitXorAssign for NumRepr<U>, bitxor_assign, overflowing_bitxor);



impl_wrapped_map_from!(impl<U:Uint> From<bool> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<u8> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<u16> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<u32> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<u64> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<u128> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<i8> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<i16> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<i32> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<i64> for NumRepr<U>);
impl_wrapped_map_from!(impl<U:Uint> From<i128> for NumRepr<U>);



impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u8);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u16);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u32);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u64);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for u128);

impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i8);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i16);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i32);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i64);
impl_wrapped_try_from_for_primitive!(impl<U:Uint> TryFrom<NumRepr<U>> for i128);


impl<U:Uint> std::cmp::Ord for NumRepr<U> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.wrapping_cmp(&other.0)
    }
}


impl<U:Uint> std::cmp::PartialOrd for NumRepr<U> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


impl<U:Uint> std::cmp::PartialEq for  NumRepr<U>  {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<U:Uint> std::cmp::Eq for NumRepr<U> {}


impl<U:Uint> std::hash::Hash for NumRepr<U> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<U:Uint> std::fmt::Debug for NumRepr<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl<U:Uint> std::fmt::Display for NumRepr<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl<U:Uint>  std::fmt::LowerHex for NumRepr<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::LowerHex::fmt(&self.0, f)
    }
}


impl<U:Uint> std::str::FromStr for NumRepr<U> {
    type Err = <U as std::str::FromStr>::Err ;

    fn from_str(value: &str) -> std::result::Result<NumRepr<U>, Self::Err> {
        Ok(<NumRepr<U>>::new(U::from_str(value)?))
    }
}

impl<U:Uint> std::convert::From<&'static str> for NumRepr<U> {
    fn from(s: &'static str) -> Self {
        <NumRepr<U>>::new(U::from(s))
    }
}




// Wrapped ops for PrimeField

impl<Fp:PrimeField> BorshSerialize for Num<Fp> {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        self.0.serialize(writer)
    }
}

impl<Fp:PrimeField> BorshDeserialize for Num<Fp> {
    fn deserialize(buf: &mut &[u8]) -> Result<Self, std::io::Error> {
        Ok(Self(Fp::deserialize(buf)?))
    }
}

impl <Fp:PrimeField> Num<Fp> {
    pub const ZERO:Self = Num(Fp::ZERO);
    pub const ONE:Self = Num(Fp::ONE);
    pub const MODULUS: NumRepr<Fp::Inner> = NumRepr(Fp::MODULUS);

    pub fn new(n:Fp) -> Self {
        Self(n)
    }

    pub fn from_uint(v:NumRepr<Fp::Inner>) -> Option<Self> {
        Some(Self(Fp::from_uint(v.0)?))
    }

    pub fn from_mont_uint(v: NumRepr<Fp::Inner>) -> Option<Self> {
        Some(Self(Fp::from_mont_uint(v.0)?))
    }

    pub fn from_uint_unchecked(v:NumRepr<Fp::Inner>) -> Self {
        Self(Fp::from_uint_unchecked(v.0))
    }

    pub fn from_mont_uint_unchecked(v: NumRepr<Fp::Inner>) -> Self {
        Self(Fp::from_mont_uint_unchecked(v.0))
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

    pub fn to_other<Fq:PrimeField>(&self) -> Option<Num<Fq>> {
        Some(Num(self.0.to_other()?))
    }

    pub fn to_other_reduced<Fq:PrimeField>(&self) -> Num<Fq> {
        Num(self.0.to_other_reduced())
    }

}



impl_wrapped_wrapping_unop!(impl <U:PrimeField> Neg for Num<U>, neg, wrapping_neg);
impl_wrapped_wrapping_binop!(impl <U:PrimeField> Add for Num<U>, add, wrapping_add);
impl_wrapped_wrapping_binop!(impl <U:PrimeField> Sub for Num<U>, sub, wrapping_sub);
impl_wrapped_wrapping_binop!(impl <U:PrimeField> Mul for Num<U>, mul, wrapping_mul);
impl_wrapped_wrapping_binop!(impl <U:PrimeField> Div for Num<U>, div, wrapping_div);

impl_wrapped_wrapping_assignop!(impl <U:PrimeField> AddAssign for Num<U>, add_assign, wrapping_add);
impl_wrapped_wrapping_assignop!(impl <U:PrimeField> SubAssign for Num<U>, sub_assign, wrapping_sub);
impl_wrapped_wrapping_assignop!(impl <U:PrimeField> MulAssign for Num<U>, mul_assign, wrapping_mul);
impl_wrapped_wrapping_assignop!(impl <U:PrimeField> DivAssign for Num<U>, div_assign, wrapping_div);



impl<Fp:PrimeField> std::cmp::PartialEq for  Num<Fp>  {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<Fp:PrimeField> std::cmp::Eq for Num<Fp> {}


impl<Fp:PrimeField> std::fmt::Debug for Num<Fp> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl<Fp:PrimeField> std::fmt::Display for Num<Fp> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}



impl<Fp:PrimeField> std::str::FromStr for Num<Fp> {
    type Err = <Fp as std::str::FromStr>::Err ;

    fn from_str(value: &str) -> std::result::Result<Num<Fp>, Self::Err> {
        Ok(<Num<Fp>>::new(Fp::from_str(value)?))
    }
}

impl<Fp:PrimeField> std::convert::From<&'static str> for Num<Fp> {
    fn from(s: &'static str) -> Self {
        <Num<Fp>>::new(Fp::from(s))
    }
}



impl<Fp:PrimeField> Serialize for Num<Fp> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>{
        Serialize::serialize(&self.to_string(), serializer)
    }
}

impl<'de, Fp:PrimeField> Deserialize<'de> for Num<Fp> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bn = <NumRepr<Fp::Inner> as Deserialize>::deserialize(deserializer)?;
        Self::from_uint(bn).ok_or(crate::serde::de::Error::custom("Field overflow"))
    }
}

