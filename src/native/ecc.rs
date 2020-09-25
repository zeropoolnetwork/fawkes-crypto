use bellman::pairing::BitIterator;
use rand::Rng;

use crate::{
    native::num::Num,
    core::field::{Field, PrimeField, PrimeFieldRepr}
};

#[derive(Clone, Copy, Debug)]
pub struct EdwardsPointEx<F: Field> {
    pub x: Num<F>,
    pub y: Num<F>,
    pub t: Num<F>,
    pub z: Num<F>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct EdwardsPoint<F: Field> {
    pub x: Num<F>,
    pub y: Num<F>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MontgomeryPoint<F: Field> {
    pub x: Num<F>,
    pub y: Num<F>,
}

pub trait JubJubParams: Sized + Clone {
    type Fr: Field;
    type Fs: Field;

    fn edwards_g(&self) -> &EdwardsPoint<Self::Fr>;

    fn edwards_d(&self) -> Num<Self::Fr>;

    fn montgomery_a(&self) -> Num<Self::Fr>;

    fn montgomery_b(&self) -> Num<Self::Fr>;

    fn montgomery_u(&self) -> Num<Self::Fr>;
}

impl<F: Field> PartialEq for EdwardsPointEx<F> {
    fn eq(&self, other: &Self) -> bool {
        self.x * other.z == other.x * self.z && self.y * other.z == other.y * self.z
    }
}

impl<F: Field> EdwardsPoint<F> {
    pub fn get_for_y<J: JubJubParams<Fr = F>>(y: Num<F>, sign: bool, params: &J) -> Option<Self> {
        let y2 = y.square();

        ((y2 - Num::one()) / (params.edwards_d() * y2 + Num::one()))
            .sqrt()
            .map(|x| {
                if x.into_inner().into_repr().is_odd() != sign {
                    Self { x: -x, y }
                } else {
                    Self { x, y }
                }
            })
    }

    pub fn subgroup_decompress<J: JubJubParams<Fr = F>>(x: Num<F>, params: &J) -> Option<Self> {
        let x2 = x.square();
        let t = ((x2 + Num::one()) / (Num::one() - params.edwards_d() * x2)).sqrt();
        match t {
            Some(y) => {
                let EdwardsPoint { x: lx, y: ly } = EdwardsPoint { x, y }
                    .into_extended()
                    .mul_raw(J::Fs::char(), params)
                    .into_affine();
                if lx.is_zero() {
                    if ly == Num::one() {
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

    pub fn rand<R: Rng, J: JubJubParams<Fr = F>>(rng: &mut R, params: &J) -> Self {
        loop {
            if let Some(p) = Self::get_for_y(rng.gen(), rng.gen(), params) {
                return p;
            }
        }
    }

    pub fn from_scalar_raw(
        t: Num<F>,
        montgomery_a: Num<F>,
        montgomery_b: Num<F>,
        montgomery_u: Num<F>,
    ) -> Self {
        fn g<F: Field>(x: Num<F>, montgomery_a: Num<F>, montgomery_b: Num<F>) -> Num<F> {
            (x.square() * (x + montgomery_a) + x) / montgomery_b
        }

        fn filter_even<F: Field>(x: Num<F>) -> Num<F> {
            if x.is_even() {
                x
            } else {
                -x
            }
        }

        let t = t + Num::one();
        let t2g1 = t.square() * montgomery_u;

        let x2 = -Num::one() / montgomery_a * (Num::one() + t2g1.inverse());

        let (mx, my) = match g(x2, montgomery_a, montgomery_b).sqrt() {
            Some(y2) => (x2, filter_even(y2)),
            _ => {
                let x3 = x2 * t2g1;
                let y3 = g(x3, montgomery_a, montgomery_b).sqrt().unwrap();
                (x3, filter_even(y3))
            }
        };

        MontgomeryPoint { x: mx, y: my }
            .into_extended()
            .mul_by_cofactor()
            .into_affine()
    }

    // assume t!= -1
    pub fn from_scalar<J: JubJubParams<Fr = F>>(t: Num<F>, params: &J) -> Self {
        Self::from_scalar_raw(
            t,
            params.montgomery_a(),
            params.montgomery_b(),
            params.montgomery_u(),
        )
    }

    pub fn zero() -> Self {
        Self {
            x: Num::zero(),
            y: Num::one(),
        }
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    pub fn mul<J: JubJubParams<Fr = F>>(&self, scalar: Num<J::Fs>, params: &J) -> Self {
        self.into_extended().mul(scalar, params).into_affine()
    }

    pub fn add<J: JubJubParams<Fr = F>>(&self, other: &Self, params: &J) -> Self {
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

    pub fn is_in_curve<J: JubJubParams<Fr = F>>(&self, params: &J) -> bool {
        // check that a point is on curve
        // y^2 - x^2 = 1 + d * x^2 * y^2

        let x2 = self.x.square();
        let y2 = self.y.square();
        y2 - x2 == Num::one() + params.edwards_d() * x2 * y2
    }

    pub fn into_montgomery(&self) -> Option<MontgomeryPoint<F>> {
        if self.x.is_zero() {
            if self.y == Num::one() {
                None
            } else {
                Some(MontgomeryPoint {
                    x: Num::zero(),
                    y: Num::zero(),
                })
            }
        } else {
            let m_x = (Num::one() + self.y) / (Num::one() - self.y);
            let m_y = m_x / self.x;
            Some(MontgomeryPoint { x: m_x, y: m_y })
        }
    }

    pub fn into_extended(&self) -> EdwardsPointEx<F> {
        let t = self.x * self.y;
        let z = Num::one();

        EdwardsPointEx {
            x: self.x,
            y: self.y,
            t,
            z,
        }
    }
}

impl<F: Field> MontgomeryPoint<F> {
    pub fn into_affine(&self) -> EdwardsPoint<F> {
        if self.x.is_zero() {
            EdwardsPoint {
                x: Num::zero(),
                y: -Num::one(),
            }
        } else {
            let e_x = self.x / self.y;
            let e_y = (self.x - Num::one()) / (self.x + Num::one());
            EdwardsPoint { x: e_x, y: e_y }
        }
    }

    pub fn into_extended(&self) -> EdwardsPointEx<F> {
        self.into_affine().into_extended()
    }
}

impl<F: Field> EdwardsPointEx<F> {
    pub fn is_in_curve<J: JubJubParams<Fr = F>>(&self, params: &J) -> bool {
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
    pub fn mul_by_cofactor(&self) -> EdwardsPointEx<F> {
        self.double().double().double()
    }

    pub fn into_affine(&self) -> EdwardsPoint<F> {
        let zinv = self.z.inverse();
        EdwardsPoint {
            x: self.x * zinv,
            y: self.y * zinv,
        }
    }

    pub fn into_montgomery(&self) -> Option<MontgomeryPoint<F>> {
        self.into_affine().into_montgomery()
    }

    //assuming in curve
    pub fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y == self.z
    }

    pub fn zero() -> Self {
        EdwardsPointEx {
            x: Num::zero(),
            y: Num::one(),
            t: Num::zero(),
            z: Num::one(),
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

    pub fn add<J: JubJubParams<Fr = F>>(&self, other: &Self, params: &J) -> Self {
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

    pub fn is_in_subgroup<J: JubJubParams<Fr = F>>(&self, params: &J) -> bool {
        self.mul_raw(J::Fs::char(), params).is_zero()
    }

    fn mul_raw<S: AsRef<[u64]>, J: JubJubParams<Fr = F>>(&self, scalar: S, params: &J) -> Self {
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

    pub fn mul<J: JubJubParams<Fr = F>>(&self, scalar: Num<J::Fs>, params: &J) -> Self {
        self.mul_raw(scalar.into_inner().into_repr(), params)
    }
}

#[cfg(test)]
mod ecc_test {
    use super::*;

    use crate::native::bn256::{Fr, Fs, JubJubBN256};
    use rand::{thread_rng, Rng};

    #[test]
    fn test_jubjubn256() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        assert!(
            jubjub_params
                .edwards_g()
                .into_extended()
                .is_in_subgroup(&jubjub_params),
            "subgroup generator should be in subgroup"
        );

        let s: Num<Fs> = rng.gen();
        let p = jubjub_params
            .edwards_g()
            .into_extended()
            .mul(s, &jubjub_params);
        assert!(
            p.is_in_subgroup(&jubjub_params),
            "point should be in subgroup"
        );

        let q = EdwardsPoint::rand(&mut rng, &jubjub_params).into_extended();
        assert!(q.add(&q, &jubjub_params) == q.double());
    }

    #[test]
    fn test_edwards_to_montgomery_and_back() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();
        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        let mp = p.into_montgomery().unwrap();
        assert!(mp.into_affine() == p, "point should be the same");
    }

    #[test]
    fn mul_by_cofactor_test() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();
        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params).into_extended();

        let p8_1 = p.mul_by_cofactor();
        let p8_2 = p.mul(num!(8), &jubjub_params);
        assert!(p8_1 == p8_2, "points should be the same");
    }
}
