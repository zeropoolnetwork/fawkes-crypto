pub mod prover;
pub mod verifier;
pub mod plonk_config;
pub mod engines;
pub mod setup;

use crate::{
    circuit::{
        cs::{RCS, CS}
    },
    core::signal::Signal,
    ff_uint::{Num, PrimeField, NumRepr},
};

use halo2_proofs::{
    arithmetic::{FieldExt},
    circuit::{AssignedCell,  Layouter, Region, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance},
    poly::kzg::commitment::ParamsKZG,
};

use self::plonk_config::PlonkConfig;
use engines::Engine;
use halo2_rand::rngs::OsRng;



pub fn num_to_halo_fp<Fx: PrimeField, Fy: FieldExt>(
    from: Num<Fx>,
) -> Fy {
    let buff = from.to_uint().into_inner();
    let buff_ref = buff.as_ref();

    let mut to = Fy::Repr::default();
    let to_ref = to.as_mut();

    assert!(buff_ref.len()*8 == to_ref.len());

    for i in 0..buff_ref.len() {
        to_ref[8*i..8*(i+1)].copy_from_slice(&buff_ref[i].to_le_bytes());
    }

    Fy::from_repr_vartime(to).unwrap()
}

pub fn halo_fp_to_num<Fx: PrimeField, Fy: FieldExt>(
    from: Fy,
) -> Num<Fx> {
    let repr = from.to_repr();
    let buff_ref = repr.as_ref();

    let mut to = NumRepr::<Fx::Inner>::ZERO;
    let to_ref = to.as_inner_mut().as_mut();

    assert!(buff_ref.len() == to_ref.len()*8);

    for i in 0..to_ref.len() {
        to_ref[i] = u64::from_le_bytes(buff_ref[8*i..8*(i+1)].try_into().unwrap());
    }

    Num::from_uint(to).unwrap()
}

pub fn num_to_halo_fp_value<Fx: PrimeField, Fy: FieldExt>(
    from: Option<Num<Fx>>,
) -> Value<Fy> {
    match from {
        Some(from)=>Value::known(num_to_halo_fp(from)),
        None=>Value::unknown(),
    }
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct HaloCS<C:CS>(RCS<C>);

impl <C:CS> HaloCS<C> {
    pub fn new(inner:RCS<C>) -> Self {
        Self(inner)
    }
}

#[derive(Clone, Debug)]
enum Halo2Cell<F:FieldExt> {
    Input(usize),
    Aux(AssignedCell<F, F>),
}

/// Assign the value specified by `val` (evaluated lazily on-demand) into the
/// cell at column `adv` and row `offset`.
///
/// This function uses `var_cells` array to keep track of previously assigned
/// witnesses. If the `val` was assigned to some cell before, this function
/// will copy it from the old locaiton ensuring the equality between the two
/// cells.
fn assign_advice_ex<
    Fr:PrimeField,
    F:FieldExt,
    AnR: Into<String>,
    An:Fn()->AnR, Val:Fn() -> Option<Num<Fr>>
>(
    region: &mut Region<F>,
    var_cells: &mut[Option::<Halo2Cell<F>>],
    annotation: An,
    offset: usize,
    adv: Column<Advice>,
    inst: Column<Instance>,
    var: usize,
    val: Val
) -> Result<(), Error> {
    if let Some(vc) = var_cells[var].as_ref() {
        match vc {
            Halo2Cell::Input(i)=> {
                region.assign_advice_from_instance(annotation, inst, *i, adv, offset)?;
            },
            Halo2Cell::Aux(cell)=> {
                cell.copy_advice(annotation, region, adv, offset)?;
            },
        }

    } else {
        let cell = region.assign_advice(
            annotation, adv, offset,
            || num_to_halo_fp_value::<_,F>(val()))?;
        var_cells[var] = Some(Halo2Cell::Aux(cell));
    };
    Ok(())
}


impl<F: FieldExt, C:CS> Circuit<F> for HaloCS<C> {
    type Config = plonk_config::PlonkConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        std::unimplemented!()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        PlonkConfig::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>
    ) -> Result<(), Error> {
        let cs = self.0.borrow();
        let num_input = cs.num_input();
        let num_var = cs.num_aux()+num_input;

        let public_indexes = cs.as_public();

        layouter.assign_region(|| format!("syntesize circuit"), |mut region| {

            let mut var_cells = vec![Option::<Halo2Cell<F>>::None; num_var];

            for i in 0..num_input {
                var_cells[public_indexes[i] as usize] = Some(Halo2Cell::Input(i));
            }

            for (offset, g) in cs.get_gate_iterator().enumerate() {
                let mut adv_helper = |ann, adv, var| {
                    assign_advice_ex(
                        &mut region, &mut var_cells,
                        || format!("assign {}[{}]", ann, offset),
                        offset, adv, config.instance, var,
                        || cs.get_value(var)
                    )
                };
                adv_helper("x", config.a, g.x)?;
                adv_helper("y", config.b, g.y)?;
                adv_helper("z", config.c, g.z)?;

                let mut fixed_helper = |ann, fix, val| {
                    region.assign_fixed(
                        || format!("assign {}[{}]", ann, offset),
                        fix, offset,
                        || num_to_halo_fp_value::<_,F>(Some(val))
                    )
                };
                fixed_helper("a", config.q_a, g.a)?;
                fixed_helper("b", config.q_b, g.b)?;
                fixed_helper("c", config.q_c, g.c)?;
                fixed_helper("d", config.q_ab, g.d)?;
                fixed_helper("e", config.constant, g.e)?;
            }

            Ok(())
        })?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Parameters<E: Engine>(pub ParamsKZG<E::BE>);

impl <E:Engine> Parameters<E> {
    pub fn setup(k:usize) -> Self {
        let params = ParamsKZG::<E::BE>::setup(k as u32, OsRng);
        Self(params)
    }
}
