use bellman::{
    SynthesisError,
    ConstraintSystem
};

use bellman::pairing::{
    Engine
};


use super::signal::{Signal, enforce};
use crate::ecc::{JubJubParams};
use crate::wrappedmath::Wrap;
use crate::circuit::mux::mux3;

#[derive(Clone)]
pub struct EdwardsPoint<E:Engine> {
    pub x: Signal<E>,
    pub y: Signal<E>
}

#[derive(Clone)]
pub struct MontgomeryPoint<E:Engine> {
    pub x: Signal<E>,
    pub y: Signal<E>
}



impl<E:Engine> EdwardsPoint<E> {
    pub fn alloc<CS:ConstraintSystem<E>>(mut cs:CS, p: Option<crate::ecc::EdwardsPoint<E>>) -> Result<Self, SynthesisError> {
        let (x_value, y_value) = match p {
            Some(p) => {
                let (x,y) = p.into_xy();
                (Some(x), Some(y))
            },
            None => (None, None)
        };

        let x = Signal::alloc(cs.namespace(|| ":=x"), x_value)?;
        let y = Signal::alloc(cs.namespace(|| ":=y"), y_value)?;
        Ok(Self {x, y})
    }

    pub fn constant(p: crate::ecc::EdwardsPoint<E>) -> Self {
        let (x, y) = p.into_xy();
        Self {x: Signal::Constant(x), y: Signal::Constant(y)}
    }


    pub fn double<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<Self, SynthesisError>{
        let v = self.x.multiply(cs.namespace(|| "xy"), &self.y)?;
        let v2 = v.square(cs.namespace(|| "x^2 y^2"))?;
        let u = (&self.x+&self.y).square(cs.namespace(|| "(x+y)^2"))?;
        Ok(Self {
            x: (&v+&v).divide(cs.namespace(|| "x3"), &(Signal::one() +  params.edwards_d()*&v2))?,
            y: (&u-&v-&v).divide(cs.namespace(|| "y3"), &(Signal::one() -  params.edwards_d()*&v2))?
        })
    }

    pub fn mul_cofactor<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<Self, SynthesisError>{
        let p2 = self.double(cs.namespace(|| "2p"), params)?;
        let p4 = p2.double(cs.namespace(|| "4p"), params)?;
        let p8 = p4.double(cs.namespace(|| "8p"), params)?;
        Ok(p8)
    }



    pub fn add<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, p: &Self, params: &J) -> Result<Self, SynthesisError> {
        let v1 = self.x.multiply(cs.namespace(|| "x1y2"), &p.y)?;
        let v2 = p.x.multiply(cs.namespace(|| "x2y1"), &self.y)?;
        let v12 = v1.multiply(cs.namespace(|| "x1y2x2y1"), &v2)?;
        let u = (&self.x+&self.y).multiply(cs.namespace(|| "(x1+y1)*(x2+y2)"), &(&p.x+&p.y))?;
        Ok(Self {
            x: (&v1+&v2).divide(cs.namespace(|| "x3"), &(Signal::one() +  params.edwards_d()*&v12))?,
            y: (&u-&v1-&v2).divide(cs.namespace(|| "y3"), &(Signal::one() -  params.edwards_d()*&v12))?
        })
    }

    pub fn assert_in_curve<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<(), SynthesisError> {
        let x2 = self.x.square(cs.namespace(|| "x^2"))?;
        let y2 = self.y.square(cs.namespace(|| "y^2"))?;

        enforce(cs.namespace(||"in_curve"), &(params.edwards_d()*&x2), &y2, &(&y2-&x2-&Signal::one()));
        Ok(())
    }

    pub fn assert_in_subgroup<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<(), SynthesisError> {
        let preimage_value = match (self.x.get_value(), self.y.get_value()) {
            (Some(x), Some(y)) => {
                let p = crate::ecc::EdwardsPoint::from_xy_unchecked(x, y);
                Some(p.mul(params.edwards_inv_cofactor().into_repr(), params))
            },
            _ => None
        };

        let preimage = EdwardsPoint::alloc(cs.namespace(|| "q"), preimage_value)?;
        preimage.assert_in_curve(cs.namespace(||"incurve"), params)?;
        let preimage8 = preimage.mul_cofactor(cs.namespace(|| "8q"), params)?;

        (&self.x - &preimage8.x).assert_zero(cs.namespace(|| "assert_x"))?;
        (&self.y - &preimage8.y).assert_zero(cs.namespace(|| "assert_y"))?;
        
        Ok(())
    }

    pub fn subgroup_decompress<CS:ConstraintSystem<E>, J:JubJubParams<E>>(mut cs:CS, x:&Signal<E>, params: &J) -> Result<Self, SynthesisError> {
        let preimage_value = match x.get_value() {
            Some(x) => {
                let p = crate::ecc::EdwardsPoint::subgroup_decompress(x, params).ok_or(SynthesisError::Unsatisfiable)?;
                Some(p.mul(params.edwards_inv_cofactor().into_repr(), params))
            },
            _ => None
        };

        let preimage = EdwardsPoint::alloc(cs.namespace(|| "q"), preimage_value)?;
        preimage.assert_in_curve(cs.namespace(||"incurve"), params)?;
        let preimage8 = preimage.mul_cofactor(cs.namespace(|| "8q"), params)?;

        (x - &preimage8.x).assert_zero(cs.namespace(|| "assert_x"))?;
        
        Ok(preimage8)
    }

    // assume nonzero subgroup point
    pub fn into_montgomery<CS:ConstraintSystem<E>>(&self, mut cs:CS) -> Result<MontgomeryPoint<E>, SynthesisError> {
        let x = (&Signal::one() + &self.y).divide(cs.namespace(|| "x3"), &(Signal::one() - &self.y))?;
        let y = x.divide(cs.namespace(|| "y3"), &self.x)?;
        Ok(MontgomeryPoint {x, y})
    }

    pub fn switch<CS:ConstraintSystem<E>>(&self, mut cs:CS, bit:&Signal<E>, if_else:&Self) -> Result<Self, SynthesisError> {
        Ok(Self {
            x: self.x.switch(cs.namespace(|| "x3"), bit, &if_else.x)?,
            y: self.y.switch(cs.namespace(|| "y3"), bit, &if_else.y)?
        })
    }

    // assume subgroup point, bits
    pub fn multiply<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, bits:&[Signal<E>], params: &J) -> Result<Self, SynthesisError> {
        fn gen_table<E:Engine, J:JubJubParams<E>>(p: &crate::ecc::EdwardsPoint<E>, params: &J) -> Vec<Vec<Wrap<E::Fr>>> {
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
        
        match (&self.x, &self.y) {        
            (&Signal::Constant(x), &Signal::Constant(y)) => {
                let mut base = crate::ecc::EdwardsPoint::from_xy_unchecked(x, y);
                if base.is_zero() {
                    Ok(EdwardsPoint {x: Signal::zero(), y: Signal::one()})
                } else {
                    let bits_len = bits.len();
                    let zeros_len = (3 - (bits_len % 3))%3;
                    let zero_bits = vec![Signal::zero(); zeros_len];
                    let all_bits = [bits, &zero_bits].concat();

                    let all_bits_len = all_bits.len();
                    let nwindows = all_bits_len / 3;

                    let mut acc = crate::ecc::EdwardsPoint::from_xy_unchecked(Wrap::zero(), Wrap::minusone());
                    
                    for _ in 0..nwindows {
                        acc = acc.add(&base, params);
                        base = base.double().double().double();
                    }

                    let (m_x, m_y) = acc.negate().into_montgomery_xy().ok_or(SynthesisError::DivisionByZero)?;

                    let mut acc = MontgomeryPoint {x: Signal::Constant(m_x), y: Signal::Constant(m_y)};
                    let mut base = crate::ecc::EdwardsPoint::from_xy_unchecked(x, y);

        
                    for i in 0..nwindows {
                        let table = gen_table(&base, params);
                        let res = mux3(cs.namespace(|| format!("mux3[{}]", i)), &all_bits[3*i..3*(i+1)], &table)?;
                        let p = MontgomeryPoint {x: res[0].clone(), y: res[1].clone()};
                        acc = acc.add(cs.namespace(|| format!("adder[{}]", i)), &p, params)?;
                        base = base.double().double().double();
                    }
                    
                    let res = acc.into_edwards(cs.namespace(|| "to_edwards"))?;
                    Ok(EdwardsPoint {x:-res.x, y:-res.y})
                }
            },
            _ => {
                let base_is_zero = self.x.is_zero(cs.namespace(|| "is_base_zero"))?;
                let dummy_point = EdwardsPoint::constant(params.edwards_g8().clone());
                let base_point = dummy_point.switch(cs.namespace(|| "dummy_switch"), &base_is_zero, self)?;

                let mut base_point = base_point.into_montgomery(cs.namespace(|| "to_montgomery"))?;
        
                let mut exponents = vec![base_point.clone()];
        
                for i in 1..bits.len() {
                    base_point = base_point.double(cs.namespace(|| format!("dlb[{}]", i)), params)?;
                    exponents.push(base_point.clone());
                }

        
                let empty_acc = MontgomeryPoint {x:Signal::zero(), y:Signal::zero()};
                let mut acc = empty_acc.clone();
        
                for i in 0..bits.len() {
                    let inc_acc = acc.add(cs.namespace(|| format!("add[{}]", i)), &exponents[i], params)?;
                    acc = inc_acc.switch(cs.namespace(|| format!("addsw[{}]", i)), &bits[i], &acc)?;
                }
        
                acc = empty_acc.switch(cs.namespace(|| "switch_empty"), &base_is_zero, &acc)?;
        
                let res = acc.into_edwards(cs.namespace(|| "to_edwards"))?;
                Ok(EdwardsPoint {x:-res.x, y:-res.y})
            }
        }
    }
}


impl<E:Engine> MontgomeryPoint<E> {
    pub fn alloc<CS:ConstraintSystem<E>>(mut cs:CS, p: Option<crate::ecc::EdwardsPoint<E>>) -> Result<Self, SynthesisError> {
        let (x_value, y_value) = match p {
            Some(p) => {
                let (x,y) = p.into_xy();
                (Some(x), Some(y))
            },
            None => (None, None)
        };
        Ok(Self {
            x: Signal::alloc(cs.namespace(|| "x"), x_value)?,
            y: Signal::alloc(cs.namespace(|| "y"), y_value)?
        })
    }

    // assume self != (0, 0)
    pub fn double<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, params: &J) -> Result<Self, SynthesisError> {
        let x2 = self.x.square(cs.namespace(|| "x^2"))?;
        let l = (Wrap::from(3u64)*&x2 + Wrap::from(2u64)*params.montgomery_a()*&self.x + &Signal::one())
            .divide(cs.namespace(|| "l"), &(Wrap::from(2u64)*params.montgomery_b() * &self.y))?;
        
        let b_l2 = params.montgomery_b()*&l.square(cs.namespace(|| "l^2"))?;
        let a = Signal::Constant(params.montgomery_a());
        
        Ok(Self {
            x: &b_l2 - &a - &self.x - &self.x,
            y: l.multiply(cs.namespace(|| "y3"), &(Wrap::from(3u64)*&self.x + &a - &b_l2))? - &self.y
        })
    }

    // assume self != p
    pub fn add<CS:ConstraintSystem<E>, J:JubJubParams<E>>(&self, mut cs:CS, p: &Self, params: &J) -> Result<Self, SynthesisError> {
        let l = (&p.y - &self.y).divide(cs.namespace(|| "l"), &(&p.x - &self.x))?;
        let b_l2 = params.montgomery_b()*&l.square(cs.namespace(|| "l^2"))?;
        let a = Signal::Constant(params.montgomery_a());
        
        Ok(Self {
            x: &b_l2 - &a - &self.x - &p.x,
            y: l.multiply(cs.namespace(|| "y3"), &(Wrap::from(2u64)*&self.x + &p.x + &a - &b_l2))? - &self.y
        })
    }

    // assume any nonzero point
    pub fn into_edwards<CS:ConstraintSystem<E>>(&self, mut cs:CS) -> Result<EdwardsPoint<E>, SynthesisError> {
        let y_is_zero = self.y.is_zero(cs.namespace(|| "is_(0,0)"))?;
        Ok(EdwardsPoint {
            x: self.x.divide(cs.namespace(|| "x3"), &(&self.y+&y_is_zero))?,
            y: (&self.x - &Signal::one()).divide(cs.namespace(|| "y3"), &(&self.x+&Signal::one()))?
        })
    }

    pub fn switch<CS:ConstraintSystem<E>>(&self, mut cs:CS, bit:&Signal<E>, if_else:&Self) -> Result<Self, SynthesisError> {
        Ok(Self {
            x: self.x.switch(cs.namespace(|| "x3"), bit, &if_else.x)?,
            y: self.y.switch(cs.namespace(|| "y3"), bit, &if_else.y)?
        })
    }
}



#[cfg(test)]
mod poseidon_test {
    use super::*;
    use sapling_crypto::circuit::test::TestConstraintSystem;
    use bellman::pairing::bn256::{Bn256, Fr};
    use rand::{Rng, thread_rng};
    use crate::ecc::{JubJubBN256, Fs};
    use crate::wrappedmath::Wrap;
    use crate::circuit::bitify::{into_bits_le_strict};


    

    #[test]
    fn test_circuit_subgroup_decompress() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let (x, y) = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params).mul(Wrap::<Fs>::from(8u64).into_repr(), &jubjub_params).into_xy();

        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_x = Signal::alloc(cs.namespace(||"x"), Some(x)).unwrap();

        let mut n_constraints = cs.num_constraints();
        let res = EdwardsPoint::subgroup_decompress(cs.namespace(||"decompress point"), &signal_x, &jubjub_params).unwrap();
        n_constraints=cs.num_constraints()-n_constraints;

        res.y.assert_constant(cs.namespace(||"check final value"), y).unwrap();

        println!("subgroup_decompress constraints = {}", n_constraints);

        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
        assert!(res.y.get_value().unwrap() == y);
    }

    #[test]
    fn test_circuit_edwards_add() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p1 = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params);
        let p2 = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params);
        
        let (p3_x, p3_y) = p1.add(&p2, &jubjub_params).into_xy();
        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_p1 = EdwardsPoint::alloc(cs.namespace(||"p1"), Some(p1)).unwrap();
        let signal_p2 = EdwardsPoint::alloc(cs.namespace(||"p2"), Some(p2)).unwrap();

        let signal_p3 = signal_p1.add(cs.namespace(||"p1+p2"), &signal_p2, &jubjub_params).unwrap();

        signal_p3.x.assert_constant(cs.namespace(||"check x"), p3_x).unwrap();
        signal_p3.y.assert_constant(cs.namespace(||"check y"), p3_y).unwrap();

        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
    }

    #[test]
    fn test_circuit_edwards_into_montgomery() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params);
        
        let (mp_x, mp_y) = p.into_montgomery_xy().unwrap();
        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_p = EdwardsPoint::alloc(cs.namespace(||"p"), Some(p)).unwrap();
        let signal_mp = signal_p.into_montgomery(cs.namespace(|| "mp")).unwrap();

        signal_mp.x.assert_constant(cs.namespace(||"check x"), mp_x).unwrap();
        signal_mp.y.assert_constant(cs.namespace(||"check y"), mp_y).unwrap();

        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
    }

    #[test]
    fn test_circuit_montgomery_into_edwards() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params);
        
        let (p_x, p_y) = p.into_xy();
        let (mp_x, mp_y) = p.into_montgomery_xy().unwrap();
        
        let mut cs = TestConstraintSystem::<Bn256>::new();

        let signal_mp = MontgomeryPoint {
            x: Signal::alloc(cs.namespace(||"mp_x"), Some(mp_x)).unwrap(),
            y: Signal::alloc(cs.namespace(||"mp_y"), Some(mp_y)).unwrap()
        };
        
        let signal_p = signal_mp.into_edwards(cs.namespace(||"p")).unwrap();

        signal_p.x.assert_constant(cs.namespace(||"check x"), p_x).unwrap();
        signal_p.y.assert_constant(cs.namespace(||"check y"), p_y).unwrap();

        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
    }


    #[test]
    fn test_circuit_montgomery_add() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p1 = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params);
        let p2 = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params);
        
        let (p3_x, p3_y) = p1.add(&p2, &jubjub_params).into_xy();
        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_p1 = EdwardsPoint::alloc(cs.namespace(||"p1"), Some(p1)).unwrap();
        let signal_p2 = EdwardsPoint::alloc(cs.namespace(||"p2"), Some(p2)).unwrap();

        let signal_mp1 = signal_p1.into_montgomery(cs.namespace(|| "mp1")).unwrap();
        let signal_mp2 = signal_p2.into_montgomery(cs.namespace(|| "mp2")).unwrap();

        let signal_mp3 = signal_mp1.add(cs.namespace(||"mp1+mp2"), &signal_mp2, &jubjub_params).unwrap();
        let signal_p3 = signal_mp3.into_edwards(cs.namespace(||"p3")).unwrap();
        
        signal_p3.x.assert_constant(cs.namespace(||"check x"), p3_x).unwrap();
        signal_p3.y.assert_constant(cs.namespace(||"check y"), p3_y).unwrap();

        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
    }

    #[test]
    fn test_circuit_montgomery_double() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params);
        
        let (p3_x, p3_y) = p.double().into_xy();
        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_p = EdwardsPoint::alloc(cs.namespace(||"p"), Some(p)).unwrap();
        let signal_mp = signal_p.into_montgomery(cs.namespace(|| "mp")).unwrap();
        let signal_mp3 = signal_mp.double(cs.namespace(||"2 mp"), &jubjub_params).unwrap();
        let signal_p3 = signal_mp3.into_edwards(cs.namespace(||"p3")).unwrap();
        
        signal_p3.x.assert_constant(cs.namespace(||"check x"), p3_x).unwrap();
        signal_p3.y.assert_constant(cs.namespace(||"check y"), p3_y).unwrap();

        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
    }


    #[test]
    fn test_circuit_edwards_mul() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params)
            .mul(Wrap::<Fs>::from(8u64).into_repr(), &jubjub_params);
        let n : Wrap<Fr> = rng.gen();
        
        let (p3_x, p3_y) = p.mul(n.into_repr(), &jubjub_params).into_xy();
        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_p = EdwardsPoint::alloc(cs.namespace(||"p"), Some(p)).unwrap();
        let signal_n = Signal::alloc(cs.namespace(||"n"), Some(n)).unwrap();

        let signal_n_bits = into_bits_le_strict(cs.namespace(|| "bitify n"), &signal_n).unwrap();

        let mut n_constraints = cs.num_constraints();
        let signal_p3 = signal_p.multiply(cs.namespace(||"p*n"), &signal_n_bits, &jubjub_params).unwrap();
        n_constraints=cs.num_constraints()-n_constraints;

        signal_p3.x.assert_constant(cs.namespace(||"check x"), p3_x).unwrap();
        signal_p3.y.assert_constant(cs.namespace(||"check y"), p3_y).unwrap();

        println!("edwards_mul constraints = {}", n_constraints);
        
        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
    }


    #[test]
    fn test_circuit_edwards_mul_const() {
        let mut rng = thread_rng();
        let jubjub_params = JubJubBN256::new();

        let p = crate::ecc::EdwardsPoint::<Bn256>::rand(&mut rng, &jubjub_params)
            .mul(Wrap::<Fs>::from(8u64).into_repr(), &jubjub_params);
        let n : Wrap<Fr> = rng.gen();
        
        let (p3_x, p3_y) = p.mul(n.into_repr(), &jubjub_params).into_xy();
        
        let mut cs = TestConstraintSystem::<Bn256>::new();
        let signal_p = EdwardsPoint::constant(p.clone()); 
        let signal_n = Signal::alloc(cs.namespace(||"n"), Some(n)).unwrap();

        let signal_n_bits = into_bits_le_strict(cs.namespace(|| "bitify n"), &signal_n).unwrap();

        let mut n_constraints = cs.num_constraints();
        let signal_p3 = signal_p.multiply(cs.namespace(||"p*n"), &signal_n_bits, &jubjub_params).unwrap();
        n_constraints=cs.num_constraints()-n_constraints;

        signal_p3.x.assert_constant(cs.namespace(||"check x"), p3_x).unwrap();
        signal_p3.y.assert_constant(cs.namespace(||"check y"), p3_y).unwrap();

        println!("edwards_mul_const constraints = {}", n_constraints);
        
        if !cs.is_satisfied() {
            let not_satisfied = cs.which_is_unsatisfied().unwrap_or("");
            assert!(false, format!("Constraints not satisfied: {}", not_satisfied));
        }
    }    
}