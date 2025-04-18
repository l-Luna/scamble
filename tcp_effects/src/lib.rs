#![feature(macro_metavar_expr)]
#![feature(macro_metavar_expr_concat)]

#![feature(portable_simd)]

use crate::effects::noise_reduction::NoiseReduction;
use crate::effects::vocoder::Vocoder;
use crate::effects::fantasy::Fantasy;

use scamble::expose_dsp_list;

pub mod effects;

expose_dsp_list!(NoiseReduction, Fantasy, Vocoder);