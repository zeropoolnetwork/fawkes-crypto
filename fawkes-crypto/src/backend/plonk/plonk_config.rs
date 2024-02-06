use halo2_curves::ff::PrimeField;
use halo2_proofs::{
    plonk::{Advice, Any, Column, ConstraintSystem, Fixed, Instance},
    poly::Rotation,
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct PlonkConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub c: Column<Advice>,
    pub q_a: Column<Fixed>,
    pub q_b: Column<Fixed>,
    pub q_c: Column<Fixed>,
    pub q_ab: Column<Fixed>,
    pub constant: Column<Fixed>,
    pub instance: Column<Instance>,
}

impl PlonkConfig {
    pub fn configure<F: PrimeField>(meta: &mut ConstraintSystem<F>) -> Self {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let c = meta.advice_column();

        let q_a = meta.fixed_column();
        let q_b = meta.fixed_column();
        let q_c = meta.fixed_column();

        let q_ab = meta.fixed_column();

        let constant = meta.fixed_column();
        let instance = meta.instance_column();

        meta.enable_equality(a);
        meta.enable_equality(b);
        meta.enable_equality(c);
        meta.enable_equality(instance);

        meta.create_gate("", |meta| {
            let [a, b, c, q_a, q_b, q_c, q_ab, constant] = [
                a.into(),
                b.into(),
                c.into(),
                q_a.into(),
                q_b.into(),
                q_c.into(),
                q_ab.into(),
                constant.into(),
            ]
            .map(|column: Column<Any>| meta.query_any(column, Rotation::cur()));

            vec![q_a * a.clone() + q_b * b.clone() + q_c * c + q_ab * a * b + constant]
        });

        PlonkConfig {
            a,
            b,
            c,
            q_a,
            q_b,
            q_c,
            q_ab,
            constant,
            instance,
        }
    }
}
