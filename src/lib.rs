#![feature(macro_metavar_expr)]
#![feature(portable_simd)]

use crate::dynamics::LocalDynamics;
use crate::noise_reduction::NoiseReduction;
use crate::windy::WindySynth;

pub mod fmod;
pub mod raw_bindings;

#[cfg(test)]
mod simulate;
mod windy;
pub mod custom_dsp;
mod result;
mod dynamics;
mod noise_reduction;

expose_dsp_list!(WindySynth, LocalDynamics, NoiseReduction);