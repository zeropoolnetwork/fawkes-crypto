use ff::{
    Field,
    PrimeField,
    SqrtField,
    PrimeFieldRepr
};

use bellman::pairing::BitIterator;

use rand::{Rng};
use bellman::pairing::bn256::{Fr};
use crate::core::num::Num;

#[derive(PrimeField)]
#[PrimeFieldModulus = "2736030358979909402780800718157159386076813972158567259200215660948447373041"]
#[PrimeFieldGenerator = "7"]
pub struct Fs(FsRepr);


#[derive(Clone, Debug)]
pub struct EdwardsPoint<F:PrimeField> {
    pub x: Num<F>,
    pub y: Num<F>,
    pub t: Num<F>,
    pub z: Num<F>
}

pub trait JubJubParams<Fr:PrimeField>: Sized {
    type Fs: PrimeField;

    fn edwards_g(&self) -> &EdwardsPoint<Fr>;

    fn edwards_d(&self) -> Num<Fr>;

    fn montgomery_a(&self) -> Num<Fr>;

    fn montgomery_b(&self) -> Num<Fr>;

    fn montgomery_u(&self) -> Num<Fr>;

    fn edwards_inv_cofactor(&self) -> Num<Self::Fs>;
}

pub struct JubJubBN256 {
    edwards_g: EdwardsPoint<Fr>,
    edwards_d: Num<Fr>,
    montgomery_a: Num<Fr>,
    montgomery_b: Num<Fr>,
    montgomery_u: Num<Fr>,
    edwards_inv_cofactor: Num<Fs>
}



impl JubJubBN256 {
    pub fn new() -> Self {
        let edwards_d = num!("12181644023421730124874158521699555681764249180949974110617291017600649128846");

        let montgomery_a = num!(168698);
        let montgomery_b = num!("21888242871839275222246405745257275088548364400416034343698204186575808326917");
        
        // value of montgomery polynomial for x=montgomery_b (has no square root in Fr)
        let montgomery_u= num!(337401);

        let edwards_g = EdwardsPoint::from_scalar_raw(Num::from_seed(b"edwards_g"), montgomery_a, montgomery_b, montgomery_u);

        
        let edwards_inv_cofactor = num!("2394026564107420727433200628387514462817212225638746351800188703329891451411");

        Self {
            edwards_g,
            edwards_d,
            montgomery_a,
            montgomery_b,
            montgomery_u,
            edwards_inv_cofactor
        }
    }
}



impl JubJubParams<Fr> for JubJubBN256 {
    type Fs = Fs;

    fn edwards_g(&self) -> &EdwardsPoint<Fr> {
        &self.edwards_g
    }


    fn edwards_d(&self) -> Num<Fr> {
        self.edwards_d
    }


    fn montgomery_a(&self) -> Num<Fr> {
        self.montgomery_a
    }

    fn montgomery_b(&self) -> Num<Fr> {
        self.montgomery_b
    }

    fn montgomery_u(&self) -> Num<Fr> {
        self.montgomery_u
    }


    fn edwards_inv_cofactor(&self) -> Num<Fs> {
        self.edwards_inv_cofactor
    }
}



impl<F: PrimeField> PartialEq for EdwardsPoint<F> {
    fn eq(&self, other: &Self) -> bool {
        self.x * other.z == other.x * self.z && self.y * other.z == other.y * self.z
    }
}

impl <F: PrimeField+SqrtField> EdwardsPoint<F> {
    pub fn get_for_y<J: JubJubParams<F>>(y: Num<F>, sign: bool, params: &J) -> Option<Self>
    {
        let y2 = y.square();
        
        ((y2 - Num::one()) / (params.edwards_d()*y2 + Num::one())).sqrt().map(|x| {
            if x.into_inner().into_repr().is_odd() != sign {
                Self::from_xy_unchecked(-x, y)
            } else {
                Self::from_xy_unchecked(x, y)
            }
        })
    }

    pub fn subgroup_decompress<J: JubJubParams<F>>(x: Num<F>, params: &J) -> Option<Self>
    {
        let x2 = x.square();
        let t = ((x2 + Num::one()) / (Num::one() - params.edwards_d()*x2 )).sqrt();
        match t {
            Some(y) => {
                let (lx, ly) = Self::from_xy_unchecked(x, y).mul_raw(J::Fs::char(), params).into_xy();
                if lx.is_zero() {
                    if ly == Num::one() {
                        Some(Self::from_xy_unchecked(x, y))
                    } else {
                        Some(Self::from_xy_unchecked(x, -y))
                    }
                } else {
                    None
                }
            },
            None => None
        }
    }

    pub fn rand<R: Rng, J: JubJubParams<F>>(rng: &mut R, params: &J) -> Self
    {
        loop {
            if let Some(p) = Self::get_for_y(rng.gen(), rng.gen(), params) {
                return p;
            }
        }
    }

    fn from_scalar_raw(t:Num<F>, montgomery_a:Num<F>, montgomery_b:Num<F>, montgomery_u:Num<F>) -> Self {
        fn g<F:PrimeField+SqrtField>(x:Num<F>, montgomery_a:Num<F>, montgomery_b:Num<F>) -> Num<F> {
            (x.square()*(x+montgomery_a)+x) / montgomery_b
        }

        fn filter_even<F:PrimeField>(x:Num<F>) -> Num<F> {
            if x.is_even() {x} else {-x}
        }

        let t = t + Num::one();
        let t2g1 = t.square()*montgomery_u;

        
        let x2 = - Num::one()/montgomery_a * (Num::one() + t2g1.inverse());

        let (mx, my) = match g(x2, montgomery_a, montgomery_b).sqrt() {
            Some(y2) => (x2, filter_even(y2)),
            _ => {
                let x3 = x2*t2g1;
                let y3 = g(x3, montgomery_a, montgomery_b).sqrt().unwrap();
                (x3, filter_even(y3))
            }
        };

        Self::from_montgomery_xy_unchecked(mx, my).mul_by_cofactor()
    }


    // assume t!= -1
    pub fn from_scalar<J: JubJubParams<F>>(t:Num<F>, params: &J) -> Self {
        Self::from_scalar_raw(t, params.montgomery_a(), params.montgomery_b(), params.montgomery_u())
    }
}


impl <F: PrimeField> EdwardsPoint<F> {
    pub fn from_xy<J: JubJubParams<F>>(x: Num<F>, y: Num<F>, params: &J) -> Option<Self>
    {
        // check that a point is on curve
        // y^2 - x^2 = 1 + d * x^2 * y^2
        
        let x2 = x.square();
        let y2 = y.square();
        if y2 - x2 != Num::one() + params.edwards_d() * x2 * y2 {
            return None
        }

        let t = x*y;
        let z = Num::one();

        Some(EdwardsPoint { x, y, t, z} )
    }

    pub fn from_xy_unchecked(x: Num<F>, y: Num<F>) -> Self
    {
        let t = x*y;
        let z = Num::one();

        EdwardsPoint { x, y, t, z}
    }






    // compress point into single E::Fr and a sign bit
    pub fn compress_into_y(&self) -> (Num<F>, bool)
    {
        // Given a y on the curve, read the x sign and leave y coordinate only
        // Important - normalize from extended coordinates
        let (x, y) = self.into_xy();
        let sign = x.into_inner().into_repr().is_odd();

        (y, sign)
    }

    /// This guarantees the point is in the prime order subgroup
    
    pub fn mul_by_cofactor(&self) -> EdwardsPoint<F>
    {
        self.double().double().double()
    }



    
    pub fn into_xy(&self) -> (Num<F>, Num<F>)
    {
        let zinv = self.z.inverse();
        (self.x*zinv, self.y*zinv)
    }

    pub fn is_zero(&self) -> bool {
        self.into_xy() == (Num::zero(), Num::one())
    }

    pub fn into_montgomery_xy(&self) -> Option<(Num<F>, Num<F>)> {
        let (e_x, e_y) = self.into_xy();

        if e_x.is_zero() {
            if e_y == Num::one() {
                None
            } else {
                Some((Num::zero(), Num::zero()))
            }
        } else {
            let m_x = (Num::one()+e_y)/(Num::one()-e_y);
            let m_y = m_x / e_x;
            Some((m_x, m_y))
        }
    }

    pub fn from_montgomery_xy_unchecked(x: Num<F>, y: Num<F>) -> Self {
        if x.is_zero() {
            Self::from_xy_unchecked(Num::zero(), -Num::one())
        } else {
            let e_x = x/y;
            let e_y = (x-Num::one())/(x+Num::one());
            Self::from_xy_unchecked(e_x, e_y)
        }
    }


    pub fn zero() -> Self {
        EdwardsPoint {
            x: Num::zero(),
            y: Num::one(),
            t: Num::zero(),
            z: Num::one()
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

    
    pub fn add<J: JubJubParams<F>>(&self, other: &Self, params:&J) -> Self
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

    pub fn is_in_subgroup<J:JubJubParams<F>>(&self, params: &J) -> bool {
        self.mul_raw(J::Fs::char(), params).is_zero()
    }


    fn mul_raw<S: AsRef<[u64]>, J: JubJubParams<F>>(
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

    pub fn mul<J: JubJubParams<F>>(
        &self,
        scalar: Num<J::Fs>,
        params: &J
    ) -> Self {
        self.mul_raw(scalar.into_inner().into_repr(), params)
    }
}


#[cfg(test)]
mod ecc_test {
    use super::*;

    use rand::{Rng, thread_rng};


    #[test]
    fn test_jubjubn256() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();
    
        assert!(jubjub_params.edwards_g().is_in_subgroup(&jubjub_params), "subgroup generator should be in subgroup");

        let s:Num<Fs> = rng.gen();
        let p = jubjub_params.edwards_g().mul(s, &jubjub_params);
        assert!(p.is_in_subgroup(&jubjub_params), "point should be in subgroup");

        let q = EdwardsPoint::rand(&mut rng, &jubjub_params);
        assert!(q.add(&q, &jubjub_params) == q.double());

        
    }

    #[test]
    fn test_edwards_to_montgomery_and_back() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();
        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        let (mx, my) = p.into_montgomery_xy().unwrap();
        assert!(EdwardsPoint::from_montgomery_xy_unchecked(mx, my) == p, "point should be the same");
    }

    #[test]
    fn mul_by_cofactor_test() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();
        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);

        let p8_1 = p.mul_by_cofactor();
        let p8_2 = p.mul(num!(8), &jubjub_params);
        assert!(p8_1 == p8_2, "points should be the same");
    }

}