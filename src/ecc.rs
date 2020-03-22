use bellman::pairing::{
    Engine,
    BitIterator
};


use ff::{
    Field,
    PrimeField,
    PrimeFieldRepr,
    SqrtField
};

use rand::{Rng};
use bellman::pairing::bn256::{Bn256, Fr};


#[derive(PrimeField)]
#[PrimeFieldModulus = "2736030358979909402780800718157159386076813972158567259200215660948447373041"]
#[PrimeFieldGenerator = "7"]
pub struct Fs(FsRepr);


#[derive(Clone)]
pub struct EdwardsPoint<E:Engine> {
    pub x: E::Fr,
    pub y: E::Fr,
    pub t: E::Fr,
    pub z: E::Fr
}

pub trait JubJubParams<E:Engine>: Sized {
    type Fs: PrimeField;

    fn edwards_g(&self) -> &EdwardsPoint<E>;

    fn edwards_g8(&self) -> &EdwardsPoint<E>;

    fn edwards_d(&self) -> &E::Fr;

    fn montgomery_a(&self) -> &E::Fr;

    fn montgomery_b(&self) -> &E::Fr;

    fn edwards_inv_cofactor(&self) -> &Fs;
}

pub struct JubJubBN256 {
    edwards_g: EdwardsPoint<Bn256>,
    edwards_g8: EdwardsPoint<Bn256>,
    edwards_d: Fr,
    montgomery_a: Fr,
    montgomery_b: Fr,
    edwards_inv_cofactor: Fs
}



impl JubJubBN256 {
    pub fn new() -> Self {
        let edwards_g = EdwardsPoint::from_xy_unchecked(
                Fr::from_str("16901293129775574849288765577905167854488686131085253343138009607974540831890").unwrap(), 
                Fr::from_str("5472060717959818805561601436314318772137091100104008585924551046643952123905").unwrap()
        );

        let edwards_g8 = EdwardsPoint::from_xy_unchecked(
            Fr::from_str("12216525397769193039033285140139874868932027386087289415053270333399021305954").unwrap(),
            Fr::from_str("16950150798460657717958625567821834550301663161624707787222815936182638968203").unwrap()
        );
       

        let edwards_d = Fr::from_str("12181644023421730124874158521699555681764249180949974110617291017600649128846").unwrap();

        let montgomery_a = Fr::from_str("168698").unwrap();
        let montgomery_b = Fr::from_str("21888242871839275222246405745257275088548364400416034343698204186575808326917").unwrap();

        
        let edwards_inv_cofactor = Fs::from_str("2394026564107420727433200628387514462817212225638746351800188703329891451411").unwrap();

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

    fn edwards_d(&self) -> &Fr {
        &self.edwards_d
    }


    fn montgomery_a(&self) -> &Fr {
        &self.montgomery_a
    }

    fn montgomery_b(&self) -> &Fr {
        &self.montgomery_b
    }

    fn edwards_inv_cofactor(&self) -> &Fs {
        &self.edwards_inv_cofactor
    }
}



impl<E: Engine> PartialEq for EdwardsPoint<E> {
    fn eq(&self, other: &Self) -> bool {
        let mut x1 = self.x;
        x1.mul_assign(&other.z);

        let mut y1 = self.y;
        y1.mul_assign(&other.z);

        let mut x2 = other.x;
        x2.mul_assign(&self.z);

        let mut y2 = other.y;
        y2.mul_assign(&self.z);

        x1 == x2 && y1 == y2
    }
}


impl <E:Engine> EdwardsPoint<E> {
    pub fn from_xy<J: JubJubParams<E>>(x: E::Fr, y: E::Fr, params: &J) -> Option<Self>
    {
        // check that a point is on curve
        // y^2 - x^2 = 1 + d * x^2 * y^2

        // tmp0 = x^2
        let mut tmp0 = x;
        tmp0.square();

        // tmp1 = y^2
        let mut tmp1 = y;
        tmp1.square();

        let mut lhs = tmp1;
        lhs.sub_assign(&tmp0);

        let mut rhs = tmp0;
        rhs.mul_assign(&tmp1);
        rhs.mul_assign(params.edwards_d());
        rhs.add_assign(&E::Fr::one());

        if rhs != lhs {
            return None;

        }

        let mut t = x;
        t.mul_assign(&y);

        Some(EdwardsPoint {
            x: x,
            y: y,
            t: t,
            z: E::Fr::one()
        })
    }

    pub fn from_xy_unchecked(x: E::Fr, y: E::Fr) -> Self
    {
        let mut t = x;
        t.mul_assign(&y);

        EdwardsPoint {
            x: x,
            y: y,
            t: t,
            z: E::Fr::one()
        }
    }

    pub fn get_for_y<J: JubJubParams<E>>(y: E::Fr, sign: bool, params: &J) -> Option<Self>
    {
        // Given a y on the curve, x^2 = (y^2 - 1) / (dy^2 + 1)
        // This is defined for all valid y-coordinates,
        // as dy^2 + 1 = 0 has no solution in Fr.

        // tmp1 = y^2
        let mut tmp1 = y;
        tmp1.square();

        // tmp2 = (y^2 * d) + 1
        let mut tmp2 = tmp1;
        tmp2.mul_assign(params.edwards_d());
        tmp2.add_assign(&E::Fr::one());

        // tmp1 = y^2 - 1
        tmp1.sub_assign(&E::Fr::one());

        match tmp2.inverse() {
            Some(tmp2) => {
                // tmp1 = (y^2 - 1) / (dy^2 + 1)
                tmp1.mul_assign(&tmp2);

                match tmp1.sqrt() {
                    Some(mut x) => {
                        if x.into_repr().is_odd() != sign {
                            x.negate();
                        }

                        let mut t = x;
                        t.mul_assign(&y);

                        Some(EdwardsPoint {
                            x: x,
                            y: y,
                            t: t,
                            z: E::Fr::one()
                        })
                    },
                    None => None
                }
            },
            None => None
        }
    }

    // compress point into single E::Fr and a sign bit
    pub fn compress_into_y(&self) -> (E::Fr, bool)
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
            let y: E::Fr = rng.gen();

            if let Some(p) = Self::get_for_y(y, rng.gen(), params) {
                return p;
            }
        }
    }

    
    pub fn into_xy(&self) -> (E::Fr, E::Fr)
    {
        let zinv = self.z.inverse().unwrap();

        let mut x = self.x;
        x.mul_assign(&zinv);

        let mut y = self.y;
        y.mul_assign(&zinv);

        (x, y)
    }

    pub fn zero() -> Self {
        EdwardsPoint {
            x: E::Fr::zero(),
            y: E::Fr::one(),
            t: E::Fr::zero(),
            z: E::Fr::one()
        }
    }


    
    pub fn negate(&self) -> Self {
        let mut p = self.clone();

        p.x.negate();
        p.t.negate();

        p
    }

    
    pub fn double(&self) -> Self {
        // See "Twisted Edwards Curves Revisited"
        //     Huseyin Hisil, Kenneth Koon-Ho Wong, Gary Carter, and Ed Dawson
        //     Section 3.3
        //     http://hyperelliptic.org/EFD/g1p/auto-twisted-extended.html#doubling-dbl-2008-hwcd

        // A = X1^2
        let mut a = self.x;
        a.square();

        // B = Y1^2
        let mut b = self.y;
        b.square();

        // C = 2*Z1^2
        let mut c = self.z;
        c.square();
        c.double();

        // D = a*A
        //   = -A
        let mut d = a;
        d.negate();

        // E = (X1+Y1)^2 - A - B
        let mut e = self.x;
        e.add_assign(&self.y);
        e.square();
        e.add_assign(&d); // -A = D
        e.sub_assign(&b);

        // G = D+B
        let mut g = d;
        g.add_assign(&b);

        // F = G-C
        let mut f = g;
        f.sub_assign(&c);

        // H = D-B
        let mut h = d;
        h.sub_assign(&b);

        // X3 = E*F
        let mut x3 = e;
        x3.mul_assign(&f);

        // Y3 = G*H
        let mut y3 = g;
        y3.mul_assign(&h);

        // T3 = E*H
        let mut t3 = e;
        t3.mul_assign(&h);

        // Z3 = F*G
        let mut z3 = f;
        z3.mul_assign(&g);

        EdwardsPoint {
            x: x3,
            y: y3,
            t: t3,
            z: z3,
        }
    }

    
    pub fn add<J: JubJubParams<E>>(&self, other: &Self, params:&J) -> Self
    {
        // See "Twisted Edwards Curves Revisited"
        //     Huseyin Hisil, Kenneth Koon-Ho Wong, Gary Carter, and Ed Dawson
        //     3.1 Unified Addition in E^e

        // A = x1 * x2
        let mut a = self.x;
        a.mul_assign(&other.x);

        // B = y1 * y2
        let mut b = self.y;
        b.mul_assign(&other.y);

        // C = d * t1 * t2
        let mut c = params.edwards_d().clone();
        c.mul_assign(&self.t);
        c.mul_assign(&other.t);

        // D = z1 * z2
        let mut d = self.z;
        d.mul_assign(&other.z);

        // H = B - aA
        //   = B + A
        let mut h = b;
        h.add_assign(&a);

        // E = (x1 + y1) * (x2 + y2) - A - B
        //   = (x1 + y1) * (x2 + y2) - H
        let mut e = self.x;
        e.add_assign(&self.y);
        {
            let mut tmp = other.x;
            tmp.add_assign(&other.y);
            e.mul_assign(&tmp);
        }
        e.sub_assign(&h);

        // F = D - C
        let mut f = d;
        f.sub_assign(&c);

        // G = D + C
        let mut g = d;
        g.add_assign(&c);

        // x3 = E * F
        let mut x3 = e;
        x3.mul_assign(&f);

        // y3 = G * H
        let mut y3 = g;
        y3.mul_assign(&h);

        // t3 = E * H
        let mut t3 = e;
        t3.mul_assign(&h);

        // z3 = F * G
        let mut z3 = f;
        z3.mul_assign(&g);

        EdwardsPoint {
            x: x3,
            y: y3,
            t: t3,
            z: z3
        }
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