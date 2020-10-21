use crate::{
    ff_uint::{BitIterBE, Num, PrimeField},
    serde::{Deserialize, Serialize},
};

#[cfg(feature = "rand_support")]
use crate::rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct EdwardsPointEx<Fr: PrimeField> {
    pub x: Num<Fr>,
    pub y: Num<Fr>,
    pub t: Num<Fr>,
    pub z: Num<Fr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct EdwardsPoint<Fr: PrimeField> {
    pub x: Num<Fr>,
    pub y: Num<Fr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MontgomeryPoint<Fr: PrimeField> {
    pub x: Num<Fr>,
    pub y: Num<Fr>,
}

pub trait JubJubParams: Sized + Clone {
    type Fr: PrimeField;
    type Fs: PrimeField;

    fn edwards_g(&self) -> &EdwardsPoint<Self::Fr>;

    fn edwards_d(&self) -> Num<Self::Fr>;

    fn montgomery_a(&self) -> Num<Self::Fr>;

    fn montgomery_b(&self) -> Num<Self::Fr>;

    fn montgomery_u(&self) -> Num<Self::Fr>;
}

impl<Fr: PrimeField> PartialEq for EdwardsPointEx<Fr> {
    fn eq(&self, other: &Self) -> bool {
        self.x * other.z == other.x * self.z && self.y * other.z == other.y * self.z
    }
}

impl<Fr: PrimeField> EdwardsPoint<Fr> {
    pub fn get_for_y<J: JubJubParams<Fr = Fr>>(y: Num<Fr>, sign: bool, params: &J) -> Option<Self> {
        let y2 = y.square();

        ((y2 - Num::ONE) / (params.edwards_d() * y2 + Num::ONE))
            .sqrt()
            .map(|x| {
                if x.is_odd() != sign {
                    Self { x: -x, y }
                } else {
                    Self { x, y }
                }
            })
    }

    pub fn subgroup_decompress<J: JubJubParams<Fr = Fr>>(x: Num<Fr>, params: &J) -> Option<Self> {
        let x2 = x.square();
        let t = ((x2 + Num::ONE) / (Num::ONE - params.edwards_d() * x2)).sqrt();
        match t {
            Some(y) => {
                let EdwardsPoint { x: lx, y: ly } = EdwardsPoint { x, y }
                    .into_extended()
                    .mul(Num::<J::Fs>::MODULUS, params)
                    .into_affine();
                if lx.is_zero() {
                    if ly == Num::ONE {
                        Some(Self { x, y })
                    } else {
                        Some(Self { x, y: -y })
                    }
                } else {
                    None
                }
            }
            None => None,
        }
    }

    #[cfg(feature = "rand_support")]
    pub fn rand<R: Rng, J: JubJubParams<Fr = Fr>>(rng: &mut R, params: &J) -> Self {
        loop {
            if let Some(p) = Self::get_for_y(rng.gen(), rng.gen(), params) {
                return p;
            }
        }
    }

    pub fn from_scalar_raw(
        t: Num<Fr>,
        montgomery_a: Num<Fr>,
        montgomery_b: Num<Fr>,
        montgomery_u: Num<Fr>,
    ) -> Self {
        fn g<Fr: PrimeField>(x: Num<Fr>, montgomery_a: Num<Fr>, montgomery_b: Num<Fr>) -> Num<Fr> {
            (x.square() * (x + montgomery_a) + x) / montgomery_b
        }

        let t = t + Num::ONE;
        let t2g1 = t.square() * montgomery_u;

        let x2 = -Num::ONE / montgomery_a * (Num::ONE + t2g1.checked_inv().unwrap());

        let (mx, my) = match g(x2, montgomery_a, montgomery_b).even_sqrt() {
            Some(y2) => (x2, y2),
            _ => {
                let x3 = x2 * t2g1;
                let y3 = g(x3, montgomery_a, montgomery_b).even_sqrt().unwrap();
                (x3, y3)
            }
        };

        MontgomeryPoint { x: mx, y: my }
            .into_extended()
            .mul_by_cofactor()
            .into_affine()
    }

    // assume t!= -1
    pub fn from_scalar<J: JubJubParams<Fr = Fr>>(t: Num<Fr>, params: &J) -> Self {
        Self::from_scalar_raw(
            t,
            params.montgomery_a(),
            params.montgomery_b(),
            params.montgomery_u(),
        )
    }

    pub fn zero() -> Self {
        Self {
            x: Num::ZERO,
            y: Num::ONE,
        }
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    pub fn mul<J: JubJubParams<Fr = Fr>>(&self, scalar: Num<J::Fs>, params: &J) -> Self {
        self.into_extended().mul(scalar, params).into_affine()
    }

    pub fn add<J: JubJubParams<Fr = Fr>>(&self, other: &Self, params: &J) -> Self {
        self.into_extended()
            .add(&other.into_extended(), params)
            .into_affine()
    }

    pub fn double(&self) -> Self {
        self.into_extended().double().into_affine()
    }

    pub fn mul_by_cofactor(&self) -> Self {
        self.into_extended().mul_by_cofactor().into_affine()
    }

    pub fn is_in_curve<J: JubJubParams<Fr = Fr>>(&self, params: &J) -> bool {
        // check that a point is on curve
        // y^2 - x^2 = 1 + d * x^2 * y^2

        let x2 = self.x.square();
        let y2 = self.y.square();
        y2 - x2 == Num::ONE + params.edwards_d() * x2 * y2
    }

    pub fn into_montgomery(&self) -> Option<MontgomeryPoint<Fr>> {
        if self.x.is_zero() {
            if self.y == Num::ONE {
                None
            } else {
                Some(MontgomeryPoint {
                    x: Num::ZERO,
                    y: Num::ZERO,
                })
            }
        } else {
            let m_x = (Num::ONE + self.y) / (Num::ONE - self.y);
            let m_y = m_x / self.x;
            Some(MontgomeryPoint { x: m_x, y: m_y })
        }
    }

    pub fn into_extended(&self) -> EdwardsPointEx<Fr> {
        let t = self.x * self.y;
        let z = Num::ONE;

        EdwardsPointEx {
            x: self.x,
            y: self.y,
            t,
            z,
        }
    }
}

impl<Fr: PrimeField> MontgomeryPoint<Fr> {
    pub fn into_affine(&self) -> EdwardsPoint<Fr> {
        if self.x.is_zero() {
            EdwardsPoint {
                x: Num::ZERO,
                y: -Num::ONE,
            }
        } else {
            let e_x = self.x / self.y;
            let e_y = (self.x - Num::ONE) / (self.x + Num::ONE);
            EdwardsPoint { x: e_x, y: e_y }
        }
    }

    pub fn into_extended(&self) -> EdwardsPointEx<Fr> {
        self.into_affine().into_extended()
    }
}

impl<Fr: PrimeField> EdwardsPointEx<Fr> {
    pub fn is_in_curve<J: JubJubParams<Fr = Fr>>(&self, params: &J) -> bool {
        // check that a point is on curve
        // Y^2 - X^2 = Z^2 + d * T^2
        // ZT == XY
        // Z!=0

        !self.z.is_zero()
            && self.z * self.t == self.x * self.y
            && self.y.square() - self.x.square()
                == self.z.square() + params.edwards_d() * self.t.square()
    }

    /// This guarantees the point is in the prime order subgroup
    pub fn mul_by_cofactor(&self) -> EdwardsPointEx<Fr> {
        self.double().double().double()
    }

    pub fn into_affine(&self) -> EdwardsPoint<Fr> {
        let zinv = self.z.checked_inv().unwrap();
        EdwardsPoint {
            x: self.x * zinv,
            y: self.y * zinv,
        }
    }

    pub fn into_montgomery(&self) -> Option<MontgomeryPoint<Fr>> {
        self.into_affine().into_montgomery()
    }

    //assuming in curve
    pub fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y == self.z
    }

    pub fn zero() -> Self {
        EdwardsPointEx {
            x: Num::ZERO,
            y: Num::ONE,
            t: Num::ZERO,
            z: Num::ONE,
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
        let e = (self.x + self.y).square() - a - b;
        let g = d + b;
        let f = g - c;
        let h = d - b;
        let x3 = e * f;
        let y3 = g * h;
        let t3 = e * h;
        let z3 = f * g;

        EdwardsPointEx {
            x: x3,
            y: y3,
            t: t3,
            z: z3,
        }
    }

    pub fn add<J: JubJubParams<Fr = Fr>>(&self, other: &Self, params: &J) -> Self {
        // See "Twisted Edwards Curves Revisited"
        //     Huseyin Hisil, Kenneth Koon-Ho Wong, Gary Carter, and Ed Dawson
        //     3.1 Unified Addition in E^e

        let a = self.x * other.x;
        let b = self.y * other.y;
        let c = params.edwards_d() * self.t * other.t;
        let d = self.z * other.z;
        let h = b + a;
        let e = (self.x + self.y) * (other.x + other.y) - h;
        let f = d - c;
        let g = d + c;
        let x3 = e * f;
        let y3 = g * h;
        let t3 = e * h;
        let z3 = f * g;

        EdwardsPointEx {
            x: x3,
            y: y3,
            t: t3,
            z: z3,
        }
    }

    pub fn is_in_subgroup<J: JubJubParams<Fr = Fr>>(&self, params: &J) -> bool {
        self.mul(Num::<J::Fs>::MODULUS, params).is_zero()
    }

    pub fn mul<S: BitIterBE, J: JubJubParams<Fr = Fr>>(&self, scalar: S, params: &J) -> Self {
        // Standard double-and-add scalar multiplication

        let mut res = Self::zero();

        for b in scalar.bit_iter_be() {
            res = res.double();

            if b {
                res = res.add(self, params);
            }
        }
        res
    }
}
