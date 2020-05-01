use ff::{Field, SqrtField, PrimeField, PrimeFieldRepr, BitIterator};
use num::bigint::{BigUint, BigInt, ToBigInt};
use num::traits::Signed;
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign, SubAssign, MulAssign, DivAssign};
use std::fmt;
use rand::{Rand, Rng};
use blake2_rfc::blake2s::Blake2s;

use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer};
use core::str::FromStr;


use crate::constants::PERSONALIZATION;



#[derive(Clone, Copy, Debug)]
pub struct Num<T:Field>(pub T);

impl<T:PrimeField> Serialize for Num<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>{
        Into::<BigUint>::into(*self).to_string().serialize(serializer)
    }
}

impl<'de, T:PrimeField> Deserialize<'de> for Num<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Num<T>, D::Error> {
        let bn = BigUint::from_str(&String::deserialize(deserializer)?).map_err(|_| de::Error::custom("Wrong number format"))?;
        
        if bn > Into::<BigUint>::into(Num::<T>::from(-1)) {
            Err(de::Error::custom("Field overflow"))
        } else {
            Ok(num!(bn))
        }
        
    }
}



impl<T:Field> fmt::Display for Num<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl<T:Field> Rand for Num<T> {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Num(rng.gen())
    }
}



impl<T:Field> Num<T> {
    #[inline]
    pub fn new(f:T) -> Self {
        Num(f)
    }

    pub fn into_bool(self) -> bool {
        if self.is_zero() {
            false
        } else if self == Num::one() {
            true
        } else {
            panic!("Boolean should be true or false")
        }
    }

    pub fn zero() -> Self {
        Num(T::zero())
    }

    pub fn one() -> Self {
        Num(T::one())
    }

    pub fn minusone() -> Self {
        -Self::one()
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        Num(self.0.inverse().expect("attempt to divide by zero"))
    }

    #[inline]
    pub fn into_inner(&self) -> T {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    #[inline]
    pub fn double(mut self) -> Self {
        self.0.double();
        self
    }

    #[inline]
    pub fn square(mut self) -> Self {
        self.0.square();
        self
    }

    #[inline]
    fn capacity(&self) -> usize {
        // for compatibility with commutative macro
        0
    }
 
}

impl<T:SqrtField> Num<T> {
    #[inline]
    pub fn sqrt(&self) -> Option<Self> {
        self.0.sqrt().map(|x| Num(x))
    }
}


#[derive(Debug, Clone)]
pub struct BitIteratorLE<E> {
    t: E,
    n: usize,
    sz: usize
}

impl<E: AsRef<[u64]>> BitIteratorLE<E> {
    pub fn new(t: E) -> Self {
        let sz = t.as_ref().len() * 64;

        BitIteratorLE { t, n:0, sz }
    }
}

impl<E: AsRef<[u64]>> Iterator for BitIteratorLE<E> {
    type Item = bool;

    fn next(&mut self) -> Option<bool> {
        if self.n == self.sz {
            None
        } else {
            let part = self.n / 64;
            let bit = self.n - (64 * part);
            self.n += 1;

            Some(self.t.as_ref()[part] & (1 << bit) > 0)
        }
    }
}



impl<T:PrimeField> Num<T> {
    fn num_bytes() -> usize {
        ((T::NUM_BITS >> 3) + if T::NUM_BITS & 7 == 0 { 0 } else { 1 }) as usize
    }

    #[inline]
    pub fn is_odd(&self) -> bool {
        self.0.into_repr().is_odd()
    }

    #[inline]
    pub fn is_even(&self) -> bool {
        self.0.into_repr().is_even()
    }

    pub fn into_binary_be(&self) -> Vec<u8> {
        let t_bytes = Self::num_bytes();
        let mut buff = vec![0u8;t_bytes];
        self.0.into_repr().write_be(&mut buff[..]).unwrap();
        buff
    }

    pub fn from_binary_be(blob: &[u8]) -> Self {
        let x = BigUint::from_bytes_be(blob); 
        Num::from(x)       
    }

    pub fn from_other<G:PrimeField>(n: Num<G>) -> Self {
        let g_bytes = Num::<G>::num_bytes();
        let mut buff = vec![0u8;g_bytes];
        n.0.into_repr().write_be(&mut buff[..]).unwrap();
        Self::from_binary_be(buff.as_ref())
    }

    pub fn into_other<G:PrimeField>(&self) -> Num<G> {
        let self_bytes = Self::num_bytes();
        let mut buff = vec![0u8;self_bytes];
        self.0.into_repr().write_be(&mut buff[..]).unwrap();
        Num::<G>::from_binary_be(buff.as_ref())
    }

    pub fn from_seed(blob: &[u8]) -> Self {
        let mut h = Blake2s::with_params(Self::num_bytes(), &[], &[], PERSONALIZATION);
        h.update(blob);
        Self::from_binary_be(h.finalize().as_ref())
    }
    
    pub fn iterbit_be(&self) -> BitIterator<T::Repr> {
        BitIterator::new(self.into_inner().into_repr())
    }

    pub fn iterbit_le(&self) -> BitIteratorLE<T::Repr> {
        BitIteratorLE::new(self.into_inner().into_repr())
    }

}



impl<T:PrimeField> Into<BigUint> for Num<T> {
    fn into(self) -> BigUint {
        let bytes = self.into_binary_be();
        BigUint::from_bytes_be(&bytes[..])
    }
}

impl<T:PrimeField> From<u64> for Num<T> {
    fn from(n: u64) -> Self {
        let mut repr = T::zero().into_raw_repr();
        repr.as_mut()[0] = n;
        Num::new(T::from_repr(repr).unwrap())
    }
}

impl<T:PrimeField> From<BigUint> for Num<T> {
    fn from(x: BigUint) -> Self {
        let t_bytes = Self::num_bytes();
        let mut order = vec![0u8;t_bytes];
        T::char().write_be(&mut order[..]).unwrap();
        let order = BigUint::from_bytes_be(order.as_ref());
        let remainder = (x % order).to_bytes_be();
        
        let mut rem_buff = vec![0u8;t_bytes];
        rem_buff[t_bytes-remainder.len()..].clone_from_slice(&remainder);
        let mut repr = T::zero().into_raw_repr();
        repr.read_be(&rem_buff[..]).unwrap();
        Num(T::from_repr(repr).unwrap())
    }
}

impl<T:PrimeField> From<BigInt> for Num<T> {
    fn from(x: BigInt) -> Self {
        let t_bytes = Self::num_bytes();
        let mut order = vec![0u8;t_bytes];
        T::char().write_be(&mut order[..]).unwrap();
        let order = BigUint::from_bytes_be(order.as_ref()).to_bigint().unwrap();
        let mut remainder = x % &order;
        if remainder.is_negative() {
            remainder+=order;
        }
        let remainder = remainder.to_biguint().unwrap().to_bytes_be();
        
        let mut rem_buff = vec![0u8;t_bytes];
        rem_buff[t_bytes-remainder.len()..].clone_from_slice(&remainder);
        let mut repr = T::zero().into_raw_repr();
        repr.read_be(&rem_buff[..]).unwrap();
        Num(T::from_repr(repr).unwrap())
    }
}


impl<T:PrimeField> From<i64> for Num<T> {
    fn from(n: i64) -> Self {
        let mut repr = T::zero().into_raw_repr();
        repr.as_mut()[0] = n.abs() as u64;
        if n >= 0 {
            Num::new(T::from_repr(repr).unwrap())
        } else {
            -Num::new(T::from_repr(repr).unwrap())
        }
    }
}


impl<T:PrimeField> From<u32> for Num<T> {
    fn from(n: u32) -> Self {
        let mut repr = T::zero().into_raw_repr();
        repr.as_mut()[0] = n as u64;
        Num::new(T::from_repr(repr).unwrap())
    }
}


impl<T:PrimeField> From<i32> for Num<T> {
    fn from(n: i32) -> Self {
        let mut repr = T::zero().into_raw_repr();
        repr.as_mut()[0] = n.abs() as u64;
        if n >= 0 {
            Num::new(T::from_repr(repr).unwrap())
        } else {
            -Num::new(T::from_repr(repr).unwrap())
        }
    }
}



impl<T:Field> From<bool> for Num<T> {
    fn from(b: bool) -> Self {
        if b {
            Num::one()
        } else {
            Num::zero()
        }
    }
}


impl<T:PrimeField> From<&str> for Num<T> {
    fn from(s: &str) -> Self {
        Num::new(T::from_str(s).unwrap())
    }
}




impl<T:Field> PartialEq for Num<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


impl<'a, T:Field> AddAssign<&'a Num<T>> for Num<T> {
    #[inline]
    fn add_assign(&mut self, other: &Num<T>) {
        self.0.add_assign(&other.0);
    }
}

impl<'a, T:Field> SubAssign<&'a Num<T>> for Num<T> {
    #[inline]
    fn sub_assign(&mut self, other: &Num<T>) {
        self.0.sub_assign(&other.0);
    }
}

impl<'a, T:Field> MulAssign<&'a Num<T>> for Num<T> {
    #[inline]
    fn mul_assign(&mut self, other: &Num<T>) {
        self.0.mul_assign(&other.0);
    }
}

impl<'a, T:Field> DivAssign<&'a Num<T>> for Num<T> {
    #[inline]
    fn div_assign(&mut self, other: &Num<T>) {
        *self *= other.inverse();
    }
}


forward_val_assign_ex!(impl<T:Field> AddAssign<Num<T>> for Num<T>, add_assign);
forward_val_assign_ex!(impl<T:Field> SubAssign<Num<T>> for Num<T>, sub_assign);
forward_val_assign_ex!(impl<T:Field> MulAssign<Num<T>> for Num<T>, mul_assign);
forward_val_assign_ex!(impl<T:Field> DivAssign<Num<T>> for Num<T>, div_assign);


impl<'a, T:Field> Add<&'a Num<T>> for Num<T> {
    type Output = Num<T>;
    #[inline]
    fn add(mut self, other: &Num<T>) -> Self::Output {
        self+=other;
        self
    }
}

impl<'a, T:Field> Mul<&'a Num<T>> for Num<T> {
    type Output = Num<T>;
    #[inline]
    fn mul(mut self, other: &Num<T>) -> Self::Output {
        self*=other;
        self
    }
}


forward_all_binop_to_val_ref_commutative_ex!(impl<T:Field> Add for Num<T>, add);
forward_all_binop_to_val_ref_commutative_ex!(impl<T:Field> Mul for Num<T>, mul);


impl<'a, T:Field> Sub<&'a Num<T>> for Num<T> {
    type Output = Num<T>;
    #[inline]
    fn sub(mut self, other: &Num<T>) -> Self::Output {
        self-=other;
        self
    }
}

impl<'a, T:Field> Div<&'a Num<T>> for Num<T> {
    type Output = Num<T>;
    #[inline]
    fn div(mut self, other: &Num<T>) -> Self::Output {
        self/=other;
        self
    }
}


forward_all_binop_to_val_ref_ex!(impl<T:Field> Sub<Num<T>> for Num<T>, sub -> Num<T>);
forward_all_binop_to_val_ref_ex!(impl<T:Field> Div<Num<T>> for Num<T>, div -> Num<T>);


impl<T:Field> Neg for Num<T> {
    type Output = Num<T>;
    #[inline]
    fn neg(mut self) -> Num<T> {
        self.0.negate();
        self
    }
}

forward_unop_ex!(impl<T:Field> Neg for Num<T>, neg);



#[cfg(test)]
mod num_test {
    use super::*;
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};


    #[test]
    fn num_test() {
        let mut rng = thread_rng();
        let order  = Into::<BigUint>::into(Num::<Fr>::minusone()) + BigUint::from(1u64);
        let a : Num<Fr> = rng.gen();
        let b : Num<Fr> = rng.gen();
        assert!(Into::<BigUint>::into(a+b) == (Into::<BigUint>::into(a) + Into::<BigUint>::into(b)) % &order);
        assert!(Into::<BigUint>::into(a*b) == (Into::<BigUint>::into(a) * Into::<BigUint>::into(b)) % &order);
        assert!(Into::<BigUint>::into(a-b) == (&order + Into::<BigUint>::into(a) - Into::<BigUint>::into(b)) % &order);
        assert!(BigUint::from(1u64) == (Into::<BigUint>::into(a.inverse()) * Into::<BigUint>::into(a)) % &order);

    }

}