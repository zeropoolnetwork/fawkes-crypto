
use crate::core::signal::{Signal};
use crate::core::abstractsignal::AbstractSignal;
use crate::core::num::Num;
use crate::core::cs::ConstraintSystem;
use crate::circuit::bitify::c_into_bits_le_strict;
use crate::circuit::mux::c_mux3;
use crate::native::ecc::{JubJubParams, EdwardsPoint, MontgomeryPoint};


use ff::{PrimeField};


#[derive(Clone)]
pub struct CEdwardsPoint<'a, CS: ConstraintSystem> {
    pub x: Signal<'a, CS>,
    pub y: Signal<'a, CS>
}


#[derive(Clone)]
pub struct CMontgomeryPoint<'a, CS: ConstraintSystem> {
    pub x: Signal<'a, CS>,
    pub y: Signal<'a, CS>
}


impl<'a, CS:ConstraintSystem> AbstractSignal<'a, CS> for CEdwardsPoint<'a, CS> {
    type Value = EdwardsPoint<CS::F>;

    #[inline]
    fn get_value(&self) -> Option<Self::Value> {
        Some(Self::Value::from_xy_unchecked(self.x.get_value()?, self.y.get_value()?))
    }

    #[inline]
    fn get_cs(&self) -> &'a CS {
        self.x.cs
    }

    fn as_const(&self) -> Option<Self::Value> {
        Some(Self::Value::from_xy_unchecked(self.x.as_const()?, self.y.as_const()?))
    }

    #[inline]
    fn from_const(cs:&'a CS, value: Self::Value) -> Self {
        let (x, y) = value.into_xy();
        Self {
            x: Signal::from_const(cs, x),
            y: Signal::from_const(cs, y)
        }
    }

    fn alloc(cs:&'a CS, value:Option<Self::Value>) -> Self {
        let (x, y) = match value {
            Some(value) => {let (x,y) = value.into_xy(); (Some(x), Some(y))},
            _ => (None, None)
        };

        Self {
            x: Signal::alloc(cs, x),
            y: Signal::alloc(cs, y)
        }
    }

    fn switch(&self, bit: &Signal<'a, CS>, if_else: &Self) -> Self {
        Self {
            x: self.x.switch(bit, &if_else.x),
            y: self.y.switch(bit, &if_else.y)
        }        
    }
}


impl<'a, CS:ConstraintSystem> AbstractSignal<'a, CS> for CMontgomeryPoint<'a, CS> {
    type Value = MontgomeryPoint<CS::F>;

    #[inline]
    fn get_value(&self) -> Option<Self::Value> {
        Some(Self::Value{x: self.x.get_value()?, y: self.y.get_value()?})
    }

    #[inline]
    fn get_cs(&self) -> &'a CS {
        self.x.cs
    }

    fn as_const(&self) -> Option<Self::Value> {
        Some(Self::Value{x:self.x.as_const()?, y:self.y.as_const()?})
    }

    #[inline]
    fn from_const(cs:&'a CS, value: Self::Value) -> Self {
        Self {
            x: Signal::from_const(cs, value.x),
            y: Signal::from_const(cs, value.y)
        }
    }

    fn alloc(cs:&'a CS, value:Option<Self::Value>) -> Self {
        Self {
            x: Signal::alloc(cs, value.map(|v| v.x)),
            y: Signal::alloc(cs, value.map(|v| v.y))
        }
    }
    fn switch(&self, bit: &Signal<'a, CS>, if_else: &Self) -> Self {
        Self {
            x: self.x.switch(bit, &if_else.x),
            y: self.y.switch(bit, &if_else.y)
        }        
    }
}



impl<'a, CS: ConstraintSystem> CEdwardsPoint<'a, CS> {

    pub fn double<J:JubJubParams<CS::F>>(&self, params: &J) -> Self{
        let v = &self.x * &self.y;
        let v2 = v.square();
        let u = (&self.x + &self.y).square();
        Self {
            x: &v*num!(2) / (Num::one() + params.edwards_d()*&v2),
            y: (&u-&v*num!(2)) / (Num::one() -  params.edwards_d()*&v2)
        }
    }

    pub fn mul_by_cofactor<J:JubJubParams<CS::F>>(&self, params: &J) -> Self {
        self.double(params).double(params).double(params)
    }



    pub fn add<J:JubJubParams<CS::F>>(&self, p: &Self, params: &J) -> Self {
        let v1 = &self.x * &p.y;
        let v2 = &p.x * &self.y;
        let v12 = &v1 * &v2;
        let u = (&self.x + &self.y) * (&p.x + &p.y);
        Self {
            x: (&v1+&v2)/ (Num::one() +  params.edwards_d()*&v12),
            y: (&u-&v1-&v2)/(Num::one() -  params.edwards_d()*&v12)
        }
    }

    pub fn assert_in_curve<J:JubJubParams<CS::F>>(&self, params: &J) {
        let x2 = self.x.square();
        let y2 = self.y.square();
        x2.cs.enforce(&(params.edwards_d()*&x2), &y2, &(&y2-&x2-Num::one()));
    }

    pub fn assert_in_subgroup<J:JubJubParams<CS::F>>(&self, params: &J) {
        let preimage_value = self.get_value().map(|p| p.mul(num!(8).inverse(), params));
        let preimage = self.derive_alloc::<Self>(preimage_value); 
        preimage.assert_in_curve(params);
        let preimage8 = preimage.mul_by_cofactor(params);

        (&self.x - &preimage8.x).assert_zero();
        (&self.y - &preimage8.y).assert_zero();
    }

    pub fn subgroup_decompress<J:JubJubParams<CS::F>>(x:&Signal<'a, CS>, params: &J) -> Self {
        let preimage_value = x.get_value()
            .map(|x| EdwardsPoint::subgroup_decompress(x, params)
            .unwrap_or(params.edwards_g().clone()).mul(num!(8).inverse(), params));
        let preimage = CEdwardsPoint::alloc(x.get_cs(), preimage_value); 
        preimage.assert_in_curve(params);
        let preimage8 = preimage.mul_by_cofactor(params);
        (x - &preimage8.x).assert_zero();
        preimage8
    }

    // assume nonzero subgroup point
    pub fn into_montgomery(&self) -> CMontgomeryPoint<'a, CS> {
        let x = (Num::one() + &self.y)/(Num::one() - &self.y);
        let y = &x / &self.x;
        CMontgomeryPoint {x, y}
    }

    // assume subgroup point, bits
    pub fn mul<J:JubJubParams<CS::F>>(&self, bits:&[Signal<'a, CS>], params: &J) -> Self {
        fn gen_table<F:PrimeField, J:JubJubParams<F>>(p: &EdwardsPoint<F>, params: &J) -> Vec<Vec<Num<F>>> {
            let mut x_col = vec![];
            let mut y_col = vec![];
            let mut q = p.clone();
            for _ in 0..8 {
                let (x, y) = q.into_montgomery_xy().unwrap();
                x_col.push(x);
                y_col.push(y);
                q = q.add(&p, params);
            }
            vec![x_col, y_col]
        }
        let cs = self.get_cs();

        match self.as_const() {        
            Some(c_base) => {
                let mut base = c_base;
                if base.is_zero() {
                    self.derive_const(EdwardsPoint::zero())
                } else {
                    let bits_len = bits.len();
                    let zeros_len = (3 - (bits_len % 3))%3;
                    let zero_bits = vec![Signal::zero(cs); zeros_len];
                    let all_bits = [bits, &zero_bits].concat();

                    let all_bits_len = all_bits.len();
                    let nwindows = all_bits_len / 3;

                    let mut acc = EdwardsPoint::from_xy_unchecked(Num::zero(), -Num::one());
                    
                    for _ in 0..nwindows {
                        acc = acc.add(&base, params);
                        base = base.double().double().double();
                    }

                    let (m_x, m_y) = acc.negate().into_montgomery_xy().unwrap();

                    let mut acc = CMontgomeryPoint::from_const(cs, MontgomeryPoint{x:m_x, y:m_y});
                    let mut base = c_base;

        
                    for i in 0..nwindows {
                        let table = gen_table(&base, params);
                        let res = c_mux3(&all_bits[3*i..3*(i+1)], &table);
                        let p = CMontgomeryPoint {x: res[0].clone(), y: res[1].clone()};
                        acc = acc.add(&p, params);
                        base = base.double().double().double();
                    }
                    
                    let res = acc.into_edwards();
                    CEdwardsPoint {x:-res.x, y:-res.y}
                }
            },
            _ => {
                let base_is_zero = self.x.is_zero();
                let dummy_point = CEdwardsPoint::from_const(cs, params.edwards_g().clone());
                let base_point = dummy_point.switch(&base_is_zero, self);

                let mut base_point = base_point.into_montgomery();
        
                let mut exponents = vec![base_point.clone()];
        
                for _ in 1..bits.len() {
                    base_point = base_point.double(params);
                    exponents.push(base_point.clone());
                }

        
                let empty_acc = CMontgomeryPoint {x:Signal::zero(cs), y:Signal::zero(cs)};
                let mut acc = empty_acc.clone();
        
                for i in 0..bits.len() {
                    let inc_acc = acc.add(&exponents[i], params);
                    acc = inc_acc.switch(&bits[i], &acc);
                }
        
                acc = empty_acc.switch(&base_is_zero, &acc);
        
                let res = acc.into_edwards();
                CEdwardsPoint {x:-res.x, y:-res.y}
            }
        }
    }

    // assuming t!=-1
    pub fn from_scalar<J:JubJubParams<CS::F>>(t:&Signal<'a, CS>, params: &J) -> Self {

        fn filter_even<F:PrimeField>(x:Num<F>) -> Num<F> {
            if x.is_even() {x} else {-x}
        }

        fn check_and_get_y<'a, CS:ConstraintSystem, J:JubJubParams<CS::F>>(x:&Signal<'a, CS>, params: &J) -> (Signal<'a, CS>, Signal<'a, CS>) {
            let g = (x.square()*(x+params.montgomery_a())+x) / params.montgomery_b();

            let preimage_value = g.get_value().map(|g| {
                match g.sqrt() {
                    Some(g_sqrt) => filter_even(g_sqrt),
                    _ => filter_even((g*params.montgomery_u()).sqrt().unwrap())
                }
            });

            let preimage = x.derive_alloc(preimage_value);
            let preimage_bits = c_into_bits_le_strict(&preimage);
            preimage_bits[0].assert_zero();

            let preimage_square = preimage.square();

            let is_square = (&g-&preimage_square).is_zero();
            let isnot_square = (&g*params.montgomery_u() - &preimage_square).is_zero();

            (&is_square+isnot_square-Num::one()).assert_zero();
            (is_square, preimage)
        }


        let t = t + Num::one();

        let t2g1 = t.square()*params.montgomery_u();
        

        let x3 = - Num::one()/params.montgomery_a() * (&t2g1 + Num::one());
        let x2 = &x3 / &t2g1;

        let (is_valid, y2) = check_and_get_y(&x2, params);        
        let (_, y3) = check_and_get_y(&x3, params);

        let x = x2.switch(&is_valid, &x3);
        let y = y2.switch(&is_valid, &y3);

        CMontgomeryPoint {x, y}.into_edwards().mul_by_cofactor(params)
    }
}


impl<'a, CS: ConstraintSystem> CMontgomeryPoint<'a, CS> {
    // assume self != (0, 0)
    pub fn double<J:JubJubParams<CS::F>>(&self, params: &J) -> Self {
        let x2 = self.x.square();
        let l = (num!(3)*&x2 + num!(2)*params.montgomery_a()*&self.x + Num::one()) / (num!(2)*params.montgomery_b() * &self.y);
        let b_l2 = params.montgomery_b()*&l.square();
        let a = params.montgomery_a();

        Self {
            x: &b_l2 - &a - num!(2) * &self.x,
            y: l*(num!(3) * &self.x + a - &b_l2) - &self.y
        }
    }

    // assume self != p
    pub fn add<J:JubJubParams<CS::F>>(&self, p: &Self, params: &J) -> Self {
        let l = (&p.y - &self.y) / (&p.x - &self.x);
        let b_l2 = params.montgomery_b()*&l.square();
        let a = params.montgomery_a();
        
        Self {
            x: &b_l2 - &a - &self.x - &p.x,
            y: l*(num!(2) * &self.x + &p.x + a - &b_l2) - &self.y
        }
    }

    // assume any nonzero point
    pub fn into_edwards(&self) -> CEdwardsPoint<'a, CS> {
        let y_is_zero = self.y.is_zero();
        CEdwardsPoint {
            x: &self.x / (&self.y + y_is_zero),
            y: (&self.x - Num::one()) / (&self.x + Num::one())
        }
    }
}



#[cfg(test)]
mod ecc_test {
    use super::*;
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};
    use crate::native::ecc::{JubJubBN256};
    use crate::circuit::bitify::{c_into_bits_le_strict};
    use crate::core::cs::TestCS;



    #[test]
    fn test_scalar_point_picker() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let t = rng.gen();

        let ref mut cs = TestCS::<Fr>::new();
        let signal_t = Signal::alloc(cs, Some(t));

        let signal_p = CEdwardsPoint::from_scalar(&signal_t, &jubjub_params);
        let (x, y) = EdwardsPoint::from_scalar(t, &jubjub_params).into_xy();

        signal_p.x.assert_const(x);
        signal_p.y.assert_const(y);
    }


    #[test]
    fn test_circuit_subgroup_decompress() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let (x, y) = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params).mul(num!(8), &jubjub_params).into_xy();

        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_x = Signal::alloc(cs, Some(x));

        let mut n_constraints = cs.num_constraints();
        let res = CEdwardsPoint::subgroup_decompress(&signal_x, &jubjub_params);
        n_constraints=cs.num_constraints()-n_constraints;

        res.y.assert_const(y);

        println!("subgroup_decompress constraints = {}", n_constraints);

        assert!(res.y.get_value().unwrap() == y);
    }

    #[test]
    fn test_circuit_edwards_add() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p1 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        let p2 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        
        let (p3_x, p3_y) = p1.add(&p2, &jubjub_params).into_xy();
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_p1 = CEdwardsPoint::alloc(cs, Some(p1));
        let signal_p2 = CEdwardsPoint::alloc(cs, Some(p2));

        let signal_p3 = signal_p1.add(&signal_p2, &jubjub_params);

        signal_p3.x.assert_const(p3_x);
        signal_p3.y.assert_const(p3_y);

    }


    #[test]
    fn test_circuit_edwards_double() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        
        let (p3_x, p3_y) = p.double().into_xy();
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_p = CEdwardsPoint::alloc(cs, Some(p));

        let signal_p3 = signal_p.double(&jubjub_params);

        signal_p3.x.assert_const(p3_x);
        signal_p3.y.assert_const(p3_y);

    }

    #[test]
    fn test_circuit_edwards_into_montgomery() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        
        let (mp_x, mp_y) = p.into_montgomery_xy().unwrap();
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_p = CEdwardsPoint::alloc(cs, Some(p));
        let signal_mp = signal_p.into_montgomery();

        signal_mp.x.assert_const(mp_x);
        signal_mp.y.assert_const(mp_y);
    }

    #[test]
    fn test_circuit_montgomery_into_edwards() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        
        let (p_x, p_y) = p.into_xy();
        let (mp_x, mp_y) = p.into_montgomery_xy().unwrap();
        
        let ref mut cs = TestCS::<Fr>::new();

        let signal_mp = CMontgomeryPoint {
            x: Signal::alloc(cs, Some(mp_x)),
            y: Signal::alloc(cs, Some(mp_y))
        };
        
        let signal_p = signal_mp.into_edwards();

        signal_p.x.assert_const(p_x);
        signal_p.y.assert_const(p_y);
    }


    #[test]
    fn test_circuit_montgomery_add() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p1 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        let p2 = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        
        let (p3_x, p3_y) = p1.add(&p2, &jubjub_params).into_xy();
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_p1 = CEdwardsPoint::alloc(cs, Some(p1));
        let signal_p2 = CEdwardsPoint::alloc(cs, Some(p2));

        let signal_mp1 = signal_p1.into_montgomery();
        let signal_mp2 = signal_p2.into_montgomery();

        let signal_mp3 = signal_mp1.add(&signal_mp2, &jubjub_params);
        let signal_p3 = signal_mp3.into_edwards();
        
        signal_p3.x.assert_const(p3_x);
        signal_p3.y.assert_const(p3_y);

    }

    #[test]
    fn test_circuit_montgomery_double() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params);
        
        let (p3_x, p3_y) = p.double().into_xy();
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_p = CEdwardsPoint::alloc(cs, Some(p));
        let signal_mp = signal_p.into_montgomery();
        let signal_mp3 = signal_mp.double(&jubjub_params);
        let signal_p3 = signal_mp3.into_edwards();
        
        signal_p3.x.assert_const(p3_x);
        signal_p3.y.assert_const(p3_y);

    }


    #[test]
    fn test_circuit_edwards_mul() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = crate::native::ecc::EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params)
            .mul(num!(8), &jubjub_params);
        let n : Num<Fr> = rng.gen();
        
        let (p3_x, p3_y) = p.mul(n.into_other(), &jubjub_params).into_xy();
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_p = CEdwardsPoint::alloc(cs, Some(p));
        let signal_n = Signal::alloc(cs, Some(n));

        let signal_n_bits = c_into_bits_le_strict(&signal_n);

        let mut n_constraints = cs.num_constraints();
        let signal_p3 = signal_p.mul(&signal_n_bits, &jubjub_params);
        n_constraints=cs.num_constraints()-n_constraints;

        signal_p3.x.assert_const(p3_x);
        signal_p3.y.assert_const(p3_y);

        println!("edwards_mul constraints = {}", n_constraints);
        
    }


    #[test]
    fn test_circuit_edwards_mul_const() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = crate::native::ecc::EdwardsPoint::<Fr>::rand(&mut rng, &jubjub_params)
            .mul(num!(8), &jubjub_params);
        let n : Num<Fr> = rng.gen();
        
        let (p3_x, p3_y) = p.mul(n.into_other(), &jubjub_params).into_xy();
        
        let ref mut cs = TestCS::<Fr>::new();
        let signal_p = CEdwardsPoint::from_const(cs, p.clone()); 
        let signal_n = Signal::alloc(cs, Some(n));

        let signal_n_bits = c_into_bits_le_strict(&signal_n);

        let mut n_constraints = cs.num_constraints();
        let signal_p3 = signal_p.mul(&signal_n_bits, &jubjub_params);
        n_constraints=cs.num_constraints()-n_constraints;

        signal_p3.x.assert_const(p3_x);
        signal_p3.y.assert_const(p3_y);

        println!("edwards_mul_const constraints = {}", n_constraints);
        
    }    
}