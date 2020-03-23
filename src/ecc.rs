use bellman::pairing::{
    Engine,
    BitIterator
};


use ff::{
    Field,
    PrimeField,
    PrimeFieldRepr
};

use rand::{Rng};
use bellman::pairing::bn256::{Bn256, Fr};
use crate::wrappedmath::Wrap;

#[derive(PrimeField)]
#[PrimeFieldModulus = "2736030358979909402780800718157159386076813972158567259200215660948447373041"]
#[PrimeFieldGenerator = "7"]
pub struct Fs(FsRepr);


#[derive(Clone)]
pub struct EdwardsPoint<E:Engine> {
    pub x: Wrap<E::Fr>,
    pub y: Wrap<E::Fr>,
    pub t: Wrap<E::Fr>,
    pub z: Wrap<E::Fr>
}

pub trait JubJubParams<E:Engine>: Sized {
    type Fs: PrimeField;

    fn edwards_g(&self) -> &EdwardsPoint<E>;

    fn edwards_g8(&self) -> &EdwardsPoint<E>;

    fn edwards_d(&self) -> Wrap<E::Fr>;

    fn montgomery_a(&self) -> Wrap<E::Fr>;

    fn montgomery_b(&self) -> Wrap<E::Fr>;

    fn edwards_inv_cofactor(&self) -> Wrap<Fs>;
}

pub struct JubJubBN256 {
    edwards_g: EdwardsPoint<Bn256>,
    edwards_g8: EdwardsPoint<Bn256>,
    edwards_d: Wrap<Fr>,
    montgomery_a: Wrap<Fr>,
    montgomery_b: Wrap<Fr>,
    edwards_inv_cofactor: Wrap<Fs>
}



impl JubJubBN256 {
    pub fn new() -> Self {
        let edwards_g = EdwardsPoint::from_xy_unchecked(
                Wrap::from("16901293129775574849288765577905167854488686131085253343138009607974540831890"),
                Wrap::from("5472060717959818805561601436314318772137091100104008585924551046643952123905")
        );

        let edwards_g8 = EdwardsPoint::from_xy_unchecked(
            Wrap::from("12216525397769193039033285140139874868932027386087289415053270333399021305954"),
            Wrap::from("16950150798460657717958625567821834550301663161624707787222815936182638968203")
        );
       

        let edwards_d = Wrap::from("12181644023421730124874158521699555681764249180949974110617291017600649128846");

        let montgomery_a = Wrap::from("168698");
        let montgomery_b = Wrap::from("21888242871839275222246405745257275088548364400416034343698204186575808326917");

        
        let edwards_inv_cofactor = Wrap::from("2394026564107420727433200628387514462817212225638746351800188703329891451411");

        Self {
            edwards_g,
            edwards_g8,
            edwards_d,
            montgomery_a,
            montgomery_b,
            edwards_inv_cofactor
        }
    }
}



impl JubJubParams<Bn256> for JubJubBN256 {
    type Fs = Fs;

    fn edwards_g(&self) -> &EdwardsPoint<Bn256> {
        &self.edwards_g
    }

    fn edwards_g8(&self) -> &EdwardsPoint<Bn256> {
        &self.edwards_g8
    }

    fn edwards_d(&self) -> Wrap<Fr> {
        self.edwards_d
    }


    fn montgomery_a(&self) -> Wrap<Fr> {
        self.montgomery_a
    }

    fn montgomery_b(&self) -> Wrap<Fr> {
        self.montgomery_b
    }

    fn edwards_inv_cofactor(&self) -> Wrap<Fs> {
        self.edwards_inv_cofactor
    }
}



impl<E: Engine> PartialEq for EdwardsPoint<E> {
    fn eq(&self, other: &Self) -> bool {
        self.x * other.z == other.x * self.z && self.y * other.z == other.y * self.z
    }
}


impl <E:Engine> EdwardsPoint<E> {
    pub fn from_xy<J: JubJubParams<E>>(x: Wrap<E::Fr>, y: Wrap<E::Fr>, params: &J) -> Option<Self>
    {
        // check that a point is on curve
        // y^2 - x^2 = 1 + d * x^2 * y^2
        
        let x2 = x.square();
        let y2 = y.square();
        if y2 - x2 != Wrap::one() + params.edwards_d() * x2 * y2 {
            return None
        }

        let t = x*y;
        let z = Wrap::one();

        Some(EdwardsPoint { x, y, t, z} )
    }

    pub fn from_xy_unchecked(x: Wrap<E::Fr>, y: Wrap<E::Fr>) -> Self
    {
        let t = x*y;
        let z = Wrap::one();

        EdwardsPoint { x, y, t, z}
    }

    pub fn get_for_y<J: JubJubParams<E>>(y: Wrap<E::Fr>, sign: bool, params: &J) -> Option<Self>
    {
        // Given a y on the curve, x^2 = (y^2 - 1) / (dy^2 + 1)
        // This is defined for all valid y-coordinates,
        // as dy^2 + 1 = 0 has no solution in Fr.

        let y2 = y.square();
        
        ((y2 - Wrap::one()) / (params.edwards_d()*y2 + Wrap::one())).sqrt().map(|x| {
            if x.into_repr().is_odd() != sign {
                Self::from_xy_unchecked(-x, y)
            } else {
                Self::from_xy_unchecked(x, y)
            }
        })
    }

    // compress point into single E::Fr and a sign bit
    pub fn compress_into_y(&self) -> (Wrap<E::Fr>, bool)
    {
        // Given a y on the curve, read the x sign and leave y coordinate only
        // Important - normalize from extended coordinates
        let (x, y) = self.into_xy();
        let sign = x.into_repr().is_odd();

        (y, sign)
    }

    /// This guarantees the point is in the prime order subgroup
    
    pub fn mul_by_cofactor(&self) -> EdwardsPoint<E>
    {
        let tmp = self.double()
                      .double()
                      .double();

        tmp
    }

    pub fn rand<R: Rng, J: JubJubParams<E>>(rng: &mut R, params: &J) -> Self
    {
        loop {
            if let Some(p) = Self::get_for_y(rng.gen(), rng.gen(), params) {
                return p;
            }
        }
    }

    
    pub fn into_xy(&self) -> (Wrap<E::Fr>, Wrap<E::Fr>)
    {
        let zinv = self.z.inverse().unwrap();
        (self.x*zinv, self.y*zinv)
    }

    pub fn is_zero(&self) -> bool {
        self.into_xy() == (Wrap::zero(), Wrap::one())
    }

    pub fn into_montgomery_xy(&self) -> Option<(Wrap<E::Fr>, Wrap<E::Fr>)> {
        let (e_x, e_y) = self.into_xy();

        if e_x.is_zero() {
            if e_y == Wrap::one() {
                None
            } else {
                Some((Wrap::zero(), Wrap::zero()))
            }
        } else {
            let m_x = (Wrap::one()+e_y)/(Wrap::one()-e_y);
            let m_y = m_x / e_x;
            Some((m_x, m_y))
        }
    }

    pub fn from_montgomery_xy_unchecked(x: Wrap<E::Fr>, y: Wrap<E::Fr>) -> Self {
        if x.is_zero() {
            Self::from_xy_unchecked(Wrap::zero(), Wrap::minusone())
        } else {
            let e_x = x/y;
            let e_y = (x-Wrap::one())/(x+Wrap::one());
            Self::from_xy_unchecked(e_x, e_y)
        }
    }


    pub fn zero() -> Self {
        EdwardsPoint {
            x: Wrap::zero(),
            y: Wrap::one(),
            t: Wrap::zero(),
            z: Wrap::one()
        }
    }


    
    pub fn negate(&self) -> Self {
        let mut p = self.clone();
        p.x = -p.x;
        p.t = -p.t;
        p
    }

    
    pub fn double(&self) -> Self {
        // See "Twisted Edwards Curves Revisited"
        //     Huseyin Hisil, Kenneth Koon-Ho Wong, Gary Carter, and Ed Dawson
        //     Section 3.3
        //     http://hyperelliptic.org/EFD/g1p/auto-twisted-extended.html#doubling-dbl-2008-hwcd

        let a = self.x.square();
        let b = self.y.square();
        let c = self.z.square().double();
        let d = -a;
        let e = (self.x+self.y).square() - a - b;
        let g = d+b;
        let f = g-c;
        let h = d-b;
        let x3 = e*f;
        let y3 = g*h;
        let t3 = e*h;
        let z3 = f*g;

        EdwardsPoint {x: x3,y: y3,t: t3, z: z3}
    }

    
    pub fn add<J: JubJubParams<E>>(&self, other: &Self, params:&J) -> Self
    {
        // See "Twisted Edwards Curves Revisited"
        //     Huseyin Hisil, Kenneth Koon-Ho Wong, Gary Carter, and Ed Dawson
        //     3.1 Unified Addition in E^e

        let a = self.x*other.x;
        let b = self.y*other.y;
        let c = params.edwards_d()*self.t*other.t;
        let d = self.z*other.z;
        let h = b+a;
        let e = (self.x+self.y)*(other.x+other.y)-h;
        let f = d-c;
        let g = d+c;
        let x3 = e*f;
        let y3 = g*h;
        let t3 = e*h;
        let z3 = f*g;

        EdwardsPoint {x: x3, y: y3, t: t3, z: z3}
    }


    pub fn mul<S: AsRef<[u64]>, J: JubJubParams<E>>(
        &self,
        scalar: S,
        params: &J
    ) -> Self
    {
        // Standard double-and-add scalar multiplication

        let mut res = Self::zero();

        for b in BitIterator::new(scalar) {
            res = res.double();

            if b {
                res = res.add(self, params);
            }
        }
        res
    }



    
}