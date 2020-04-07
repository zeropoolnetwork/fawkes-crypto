use ff::{Field, SqrtField, PrimeField, PrimeFieldRepr};
use num::bigint::{BigUint};
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign, SubAssign, MulAssign, DivAssign};
use std::default::Default;
use std::fmt;
use rand::{Rand, Rng};

#[derive(Clone, Copy, Debug)]
pub struct Num<T:Field>(T);


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



impl<T:PrimeField> Num<T> {
    fn num_bytes() -> usize {
        ((T::NUM_BITS >> 3) + if T::NUM_BITS & 7 == 0 { 0 } else { 1 }) as usize
    }

    pub fn into_binary_be(&self) -> Vec<u8> {
        let t_bytes = Self::num_bytes();
        let mut buff = vec![0u8;t_bytes];
        self.0.into_repr().write_be(&mut buff[..]).unwrap();
        buff
    }

    pub fn from_binary_be(blob: &[u8]) -> Self {        
        let t_bytes = Self::num_bytes();
        let mut order = vec![0u8;t_bytes];
        T::char().write_be(&mut order[..]).unwrap();
        let order = BigUint::from_bytes_be(order.as_ref());
        let x = BigUint::from_bytes_be(blob);
        let remainder = (x % order).to_bytes_be();
        
        let mut rem_buff = vec![0u8;t_bytes];
        rem_buff[t_bytes-remainder.len()..].clone_from_slice(&remainder);
        let mut repr = T::zero().into_raw_repr();
        repr.read_be(&rem_buff[..]).unwrap();
        Num(T::from_repr(repr).unwrap())
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

impl<T:Field> Default for Num<T> {
    #[inline]
    fn default() -> Self {
        Num(T::zero())
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


forward_val_assign_ex!(impl<T:Field> AddAssign for Num<T>, add_assign);
forward_val_assign_ex!(impl<T:Field> SubAssign for Num<T>, sub_assign);
forward_val_assign_ex!(impl<T:Field> MulAssign for Num<T>, mul_assign);
forward_val_assign_ex!(impl<T:Field> DivAssign for Num<T>, div_assign);


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


forward_all_binop_to_val_ref_ex!(impl<T:Field> Sub for Num<T>, sub);
forward_all_binop_to_val_ref_ex!(impl<T:Field> Div for Num<T>, div);


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