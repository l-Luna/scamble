#![feature(macro_metavar_expr)]
#![feature(portable_simd)]
#![feature(panic_payload_as_str)]

use crate::fantasy::Fantasy;
use crate::noise_reduction::NoiseReduction;
use crate::vocoder::Vocoder;

pub mod fmod;
pub mod raw_bindings;

#[cfg(test)]
pub mod simulate; // accessed in test mod by noise_reduction
mod windy;
pub mod custom_dsp;
mod result;
mod dynamics;
mod noise_reduction;
mod exact;
mod fantasy;
mod vocoder;

expose_dsp_list!(NoiseReduction, Fantasy, Vocoder);