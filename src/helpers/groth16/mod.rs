pub mod prover;
pub mod verifier;
pub mod ethereum;

use std::cell::RefCell;
use crate::core::cs::{ConstraintSystem};
use crate::native::num::Num;
use crate::circuit::num::{CNum, Index};
use crate::core::field::{Field, PrimeField, AbstractField};


use bellman::{self, SynthesisError};
use pairing::bn256::{Fq, FqRepr, Fq2, G1Affine, G2Affine};

use std::mem::transmute;
use lazy_static::lazy_static;

lazy_static! { 
    static ref B_COEFF: Fq = Fq::from_raw_repr(FqRepr([
        0x7a17caa950ad28d7,
        0x1f6ac17ae15521b9,
        0x334bea4e696bd284,
        0x2a1f6744ce179d8e,
    ])).unwrap();
}

lazy_static! { 
    static ref B_COEFF_FQ2: Fq2 = Fq2 {
        c0: Fq::from_raw_repr(FqRepr([
            0x3bf938e377b802a8,
            0x020b1b273633535d,
            0x26b7edf049755260,
            0x2514c6324384a86d,
        ])).unwrap(),
        c1: Fq::from_raw_repr(FqRepr([
            0x38e7ecccd1dcff67,
            0x65f0b37d93ce0d3e,
            0xd749d0dd22ac00aa,
            0x0141b9ce4a688d4d,
        ])).unwrap(),
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize="", deserialize=""))]
pub struct G1PointData<F:Field>(Num<F>, Num<F>);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize="", deserialize=""))]
pub struct G2PointData<F:Field>((Num<F>, Num<F>), (Num<F>, Num<F>));

fn is_on_curve<F:AbstractField>(x:F, y:F, is_zero:bool, b:&F) -> bool {
    if is_zero {
        true
    } else {
        // Check that the point is on the curve
        let mut y2 = y;
        y2.square();

        let mut x3b = x;
        x3b.square();
        x3b.mul_assign(&x);
        x3b.add_assign(b);
        y2 == x3b
    }
}


impl From<G2Affine> for G2PointData<Fq> {
    fn from(p: G2Affine) -> Self {
        let (x, y, f) = unsafe{ transmute::<_,(Fq2,Fq2,bool)>(p)};
        if f {
            Self((num!(0), num!(0)), (num!(0), num!(0)))
        } else {
            //use big endian here 
            Self((Num(x.c1), Num(x.c0)),(Num(y.c1), Num(y.c0)))
        }
    }
}

impl Into<G2Affine> for G2PointData<Fq> {
    fn into(self) -> G2Affine {
        //use big endian here
        let x = Fq2{c0:self.0 .1.into_inner(), c1: self.0 .0.into_inner()};
        let y = Fq2{c0:self.1 .1.into_inner(), c1: self.1 .0.into_inner()};
        let is_zero = x.is_zero() && y.is_zero();
        assert!(is_on_curve(x, y, is_zero, &B_COEFF_FQ2), "point should be in curve");
        unsafe {transmute::<_,G2Affine>((x,y,is_zero))}
    }
}

impl From<G1Affine> for G1PointData<Fq> {
    fn from(p: G1Affine) -> Self {
        let (x, y, f) = unsafe{ transmute::<_,(Fq,Fq,bool)>(p)};
        if f {
            Self(num!(0), num!(0))
        } else {
            Self(Num(x), Num(y))
        }
    }
}

impl Into<G1Affine> for G1PointData<Fq> {
    fn into(self) -> G1Affine {
        let x = self.0.into_inner();
        let y = self.1.into_inner();
        let is_zero = x.is_zero() && y.is_zero();
        assert!(is_on_curve(x, y, is_zero, &B_COEFF), "point should be in curve");
        unsafe {transmute::<_,G1Affine>((x,y,is_zero))}
    }
}

pub trait Assignment<T> {
    fn get(&self) -> Result<&T, SynthesisError>;
    fn grab(self) -> Result<T, SynthesisError>;
}

impl<T: Clone> Assignment<T> for Option<T> {
    fn get(&self) -> Result<&T, SynthesisError> {
        match *self {
            Some(ref v) => Ok(v),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }

    fn grab(self) -> Result<T, SynthesisError> {
        match self {
            Some(v) => Ok(v),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }
}

impl<T: Field> Assignment<T> for Option<Num<T>> {
    fn get(&self) -> Result<&T, SynthesisError> {
        match self {
            Some(ref v) => Ok(&v.0),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }

    fn grab(self) -> Result<T, SynthesisError> {
        match self {
            Some(v) => Ok(v.into_inner()),
            None => Err(SynthesisError::AssignmentMissing)
        }
    }
}


pub struct Groth16CS<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> {
    pub ninputs:RefCell<usize>,
    pub naux:RefCell<usize>,
    pub ncons:RefCell<usize>,
    pub bcs:RefCell<BCS>,
    be: std::marker::PhantomData<BE>
}

impl<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> Clone for Groth16CS<BE, BCS> {
    fn clone(&self) -> Self {
        panic!("Clone is not implemented for Groth16CS")
    }
}


impl<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>>  Groth16CS<BE, BCS> {
    pub fn new(cs:BCS) -> Self {
        Self {
            ninputs: RefCell::new(1),
            naux: RefCell::new(0),
            ncons: RefCell::new(0),
            bcs: RefCell::new(cs),
            be: std::marker::PhantomData
        }

    }
}


impl<BE:bellman::pairing::Engine, BCS: bellman::ConstraintSystem<BE>> ConstraintSystem for Groth16CS<BE, BCS> {
    type F = BE::Fr;

    fn alloc(&self, value: Option<Num<Self::F>>) -> Index {
        let mut naux_ref = self.ninputs.borrow_mut();
        let naux = *naux_ref;
        *naux_ref+=1;
        self.bcs.borrow_mut().alloc(||format!("a[{}]", naux), || value.grab()).map(|e| unsafe{std::mem::transmute(e)}).unwrap()
            
    }
    fn alloc_input(&self, value: Option<Num<Self::F>>) -> Index {
        let mut ninputs_ref = self.ninputs.borrow_mut();
        let ninputs = *ninputs_ref;
        *ninputs_ref+=1;
        self.bcs.borrow_mut().alloc_input(||format!("i[{}]", ninputs), || value.grab()).map(|e| unsafe{std::mem::transmute(e)}).unwrap()
    }

    fn enforce(&self, a:&CNum<Self>, b:&CNum<Self>, c:&CNum<Self>) {
        fn into_bellman_lc<BE:bellman::pairing::Engine, CS:ConstraintSystem>(s:&CNum<CS>) -> bellman::LinearCombination<BE> {
                let res = s.lc.iter().map(|(k, v)| (*k, v.into_inner())).collect::<Vec<_>>();
                unsafe {std::mem::transmute(res)}
        }
        
        let mut ncons_ref = self.ncons.borrow_mut();
        let ncons = *ncons_ref;
        *ncons_ref += 1;
        let a = into_bellman_lc(a);
        let b = into_bellman_lc(b);
        let c = into_bellman_lc(c);
        self.bcs.borrow_mut().enforce(|| format!("c[{}]", ncons), |_| a, |_| b, |_| c);
    }
}
