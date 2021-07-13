#[cfg(feature = "r1cs-file")]
pub use self::const_tracker::*;
#[cfg(feature = "r1cs-file")]
pub use self::r1cs_file::*;
#[cfg(feature = "wtns-file")]
pub use self::wtns_file::*;

#[cfg(feature = "r1cs-file")]
mod const_tracker;
#[cfg(feature = "r1cs-file")]
mod r1cs_file;
#[cfg(feature = "wtns-file")]
mod wtns_file;