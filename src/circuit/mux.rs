use bellman::pairing::{
    Engine
};


use bellman::{
    SynthesisError,
    ConstraintSystem
};

use super::signal::Signal;
use crate::wrappedmath::Wrap;


// this method is described here https://iden3.readthedocs.io/en/latest/iden3_repos/research/publications/zkproof-standards-workshop-2/pedersen-hash/pedersen.html

pub fn mux3<E:Engine, CS:ConstraintSystem<E>>(mut cs:CS, s:&[Signal<E>], c:&[Vec<Wrap<E::Fr>>]) -> Result<Vec<Signal<E>>, SynthesisError> {
    assert!(s.len()==3, "should be 3 bits");
    for i in 0..c.len() {
        assert!(c[i].len() == 8, "should be 8 constants");
    }

    let s10 = s[0].multiply(cs.namespace(|| "compute s10"), &s[1])?;
    let mut res = vec![];

    for i in 0..c.len() {
        let a210 = (c[i][7]-c[i][6]-c[i][5]+c[i][4] - c[i][3]+c[i][2]+c[i][1]-c[i][0]) * &s10;
        let a21 = (c[i][6]-c[i][4]-c[i][2]+c[i][0]) * &s[1];
        let a20 = (c[i][5]-c[i][4]-c[i][1]+c[i][0]) * &s[0];
        let a2 =  Signal::Constant(c[i][4]-c[i][0]);

        let a10 = (c[i][3]-c[i][2]-c[i][1]+c[i][0]) * &s10;
        let a1 = (c[i][2]-c[i][0]) * &s[1];
        let a0 = (c[i][1]-c[i][0]) * &s[0];
        let a = Signal::Constant(c[i][0]);

        res.push( (&a210 + &a21 + &a20 + &a2).multiply(cs.namespace(|| format!("{}th selector", i)), &s[2])? + &a10 +  &a1 +  &a0 +  &a);
    }

    Ok(res)
}
