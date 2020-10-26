#[cfg(not(feature = "borsh_support"))]
pub trait Borsh {}

#[cfg(feature = "borsh_support")]
pub trait Borsh: borsh::BorshSerialize + borsh::BorshDeserialize {}

#[cfg(not(feature = "borsh_support"))]
impl<T> Borsh for T {}

#[cfg(feature = "borsh_support")]
impl<T> Borsh for T where T: borsh::BorshSerialize + borsh::BorshDeserialize {}
