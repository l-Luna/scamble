#![feature(panic_payload_as_str)]
#![feature(macro_metavar_expr)]
#![feature(macro_metavar_expr_concat)]

#[cfg(feature = "fmod_bindings")]
pub mod fmod;
#[cfg(all(test, feature = "fmod_bindings"))]
pub mod simulate;

pub mod dsp;
pub mod raw_bindings;
pub mod result;
pub mod data;