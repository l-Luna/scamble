#![feature(macro_metavar_expr)]
#![feature(portable_simd)]
#![feature(panic_payload_as_str)]

use effects::fantasy::Fantasy;
use effects::noise_reduction::NoiseReduction;
use effects::vocoder::Vocoder;
use effects::cassette_player::CassettePlayer;

#[cfg(feature = "fmod_bindings")]
pub mod fmod;
#[cfg(all(test, feature = "fmod_bindings"))]
pub mod simulate;

pub mod dsp;
pub mod effects;
pub mod raw_bindings;
pub mod result;
pub mod data;

expose_dsp_list!(NoiseReduction, Fantasy, Vocoder, CassettePlayer);
