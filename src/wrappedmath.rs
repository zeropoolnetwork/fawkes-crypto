use ff::{
    Field,
    PrimeField,
    PrimeFieldRepr,
    SqrtField
};

use rand::{Rand, Rng};
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign, SubAssign, MulAssign, DivAssign};
use num::bigint::{BigUint};

#[derive(Copy, Clone)]
pub struct Wrap<T:Field>(pub T);

impl<T:Field> Wrap<T> {
    pub fn new(f:T) -> Self {
        Wrap(f)
    }

    pub fn zero() -> Self {
        Wrap(T::zero())
    }

    pub fn one() -> Self {
        Wrap(T::one())
    }

    pub fn minusone() -> Self {
        -Wrap(T::one())
    }

    pub fn inverse(&self) -> Option<Self> {
        self.0.inverse().map(|e| Wrap(e))
    }

    pub fn into_inner(&self) -> T {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn double(&self) -> Self {
        let mut t = self.0.clone();
        t.double();
        Wrap(t)
    }

    pub fn negate(&self) -> Self {
        let mut t = self.0.clone();
        t.negate();
        Wrap(t)
    }

    pub fn square(&self) -> Self {
        let mut t = self.0.clone();
        t.square();
        Wrap(t)
    }

}

impl<T:SqrtField> Wrap<T> {
    pub fn sqrt(self) -> Option<Self> {
        self.0.sqrt().map(|x| Wrap(x))
    }
}

impl<T:PrimeField> Wrap<T> {
    fn num_bytes() -> usize {
        ((T::NUM_BITS >> 3) + if T::NUM_BITS & 7 == 0 { 0 } else { 1 }) as usize
    }

    pub fn into_repr(&self) -> T::Repr {
        self.0.into_repr()
    }

    pub fn into_binary_be(&self) -> Vec<u8> {
        let t_bytes = Self::num_bytes();
        let mut buff = vec![0u8;t_bytes];
        self.into_repr().write_be(&mut buff[..]).unwrap();
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
        Wrap(T::from_repr(repr).unwrap())
    }

    pub fn from_other<G:PrimeField>(n: Wrap<G>) -> Self {
        let g_bytes = Wrap::<G>::num_bytes();
        let mut buff = vec![0u8;g_bytes];
        n.0.into_repr().write_be(&mut buff[..]).unwrap();
        Self::from_binary_be(buff.as_ref())
    }
}


impl<T:Field> PartialEq for Wrap<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T:PrimeField> From<u64> for Wrap<T> {
    fn from(n: u64) -> Self {
        let mut repr = T::zero().into_raw_repr();
        repr.as_mut()[0] = n;
        Wrap::new(T::from_repr(repr).unwrap())
    }
}



impl<T:PrimeField> From<bool> for Wrap<T> {
    fn from(b: bool) -> Self {
        if b {
            Wrap::one()
        } else {
            Wrap::zero()
        }
    }
}


impl<T:PrimeField> From<&str> for Wrap<T> {
    fn from(s: &str) -> Self {
        Wrap::new(T::from_str(s).unwrap())
    }
}


impl<T:PrimeField> From<i64> for Wrap<T> {
    fn from(n: i64) -> Self {
        let mut repr = T::zero().into_raw_repr();
        repr.as_mut()[0] = n.abs() as u64;
        if n >= 0 {
            Wrap::new(T::from_repr(repr).unwrap())
        } else {
            -Wrap::new(T::from_repr(repr).unwrap())
        }
    }
}





impl<T:Field> Add<Wrap<T>> for Wrap<T> {
    type Output = Wrap<T>;

    fn add(self, other: Wrap<T>) -> Self::Output {
        let mut res = self;
        res.0.add_assign(&other.0);
        res
    }
}

impl<T:Field> AddAssign<Wrap<T>> for Wrap<T> {

    fn add_assign(&mut self, other: Wrap<T>) {
        self.0.add_assign(&other.0);
    }
}


impl<T:Field> Sub<Wrap<T>> for Wrap<T> {
    type Output = Wrap<T>;

    fn sub(self, other: Wrap<T>) -> Self::Output {
        let mut res = self;
        res.0.sub_assign(&other.0);
        res
    }
}

impl<T:Field> SubAssign<Wrap<T>> for Wrap<T> {

    fn sub_assign(&mut self, other: Wrap<T>) {
        self.0.sub_assign(&other.0);
    }
}


impl<T:Field> Mul<Wrap<T>> for Wrap<T> {
    type Output = Wrap<T>;

    fn mul(self, other: Wrap<T>) -> Self::Output {
        let mut res = self;
        res.0.mul_assign(&other.0);
        res
    }
}

impl<T:Field> MulAssign<Wrap<T>> for Wrap<T> {

    fn mul_assign(&mut self, other: Wrap<T>) {
        self.0.mul_assign(&other.0);
    }
}



impl<T:Field> Div<Wrap<T>> for Wrap<T> {
    type Output = Wrap<T>;

    fn div(self, other: Wrap<T>) -> Self::Output {
        let mut res = self;
        res.0.mul_assign(&other.0.inverse().unwrap());
        res
    }
}

impl<T:Field> DivAssign<Wrap<T>> for Wrap<T> {

    fn div_assign(&mut self, other: Wrap<T>) {
        self.0.mul_assign(&other.0.inverse().unwrap());
    }
}



impl<T:Field> Neg for Wrap<T> {
    type Output = Wrap<T>;

    fn neg(self) -> Self::Output {
        let mut res = self;
        res.0.negate();
        res
    }
}


impl<T:Field> Rand for Wrap<T> {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Wrap(rng.gen())
    }
}
