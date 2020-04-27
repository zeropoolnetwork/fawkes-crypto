use crate::core::signal::CNum;
use crate::core::num::Num;
use crate::core::cs::ConstraintSystem;

// this method is described here https://iden3.readthedocs.io/en/latest/iden3_repos/research/publications/zkproof-standards-workshop-2/pedersen-hash/pedersen.html

pub fn c_mux3<'a, CS:ConstraintSystem>(s:&[CNum<'a, CS>], c:&[Vec<Num<CS::F>>]) -> Vec<CNum<'a, CS>> {
    assert!(s.len()==3, "should be 3 bits");
    for i in 0..c.len() {
        assert!(c[i].len() == 8, "should be 8 constants");
    }

    let s10 = &s[0] * &s[1];
    let mut res = vec![];

    for i in 0..c.len() {
        let a210 = (c[i][7]-c[i][6]-c[i][5]+c[i][4] - c[i][3]+c[i][2]+c[i][1]-c[i][0]) * &s10;
        let a21 = (c[i][6]-c[i][4]-c[i][2]+c[i][0]) * &s[1];
        let a20 = (c[i][5]-c[i][4]-c[i][1]+c[i][0]) * &s[0];
        let a2 =  c[i][4]-c[i][0];

        let a10 = (c[i][3]-c[i][2]-c[i][1]+c[i][0]) * &s10;
        let a1 = (c[i][2]-c[i][0]) * &s[1];
        let a0 = (c[i][1]-c[i][0]) * &s[0];
        let a = c[i][0];

        res.push( (a210 + a21 + a20 + a2)* &s[2] + a10 +  a1 +  a0 +  a);
    }
    res
}