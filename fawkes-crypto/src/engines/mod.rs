use ff_uint::construct_uint;

pub mod bls12_381;
pub mod bn256;

construct_uint! {
    struct _U256(4);
}

construct_uint! {
    struct _U384(6);
}

pub type U256 = _U256;
pub type U384 = _U384;
