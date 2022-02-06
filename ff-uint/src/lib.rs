#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "borsh_support")]
#[doc(hidden)]
pub use borsh;
#[doc(hidden)]
pub use byteorder;
#[doc(hidden)]
pub use concat_idents::concat_idents;
#[doc(hidden)]
pub use crunchy::unroll;
#[doc(hidden)]
#[cfg(feature = "rand_support")]
pub use rand;
#[doc(hidden)]
pub use rustc_hex;
#[cfg(feature = "serde_support")]
#[doc(hidden)]
pub use serde;
#[cfg(feature = "scale_support")]
#[doc(hidden)]
pub use parity_scale_codec;
#[cfg(feature = "scale_support")]
#[doc(hidden)]
pub use parity_scale_codec_derive;
#[cfg(feature = "scale_support")]
#[doc(hidden)]
pub use scale_info;
#[doc(hidden)]
pub use static_assertions;

#[macro_use]
mod uint;
#[macro_use]
mod ff;
mod num;
mod traits;


#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std as alloc;

pub extern crate seedbox;

pub use ff::traits::*;
pub use ff::*;
pub use num::*;
pub use uint::macros::*;
pub use uint::traits::*;

#[cfg(feature = "std")]
pub mod maybestd {
    pub use std::{
        borrow, string, vec, format, boxed, rc, sync
    };
}

#[cfg(not(feature = "std"))]
pub mod maybestd {
    pub use alloc::{
        borrow, string, vec, format, boxed, rc, sync
    };
}
