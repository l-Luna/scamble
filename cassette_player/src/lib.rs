#![feature(macro_metavar_expr)]
#![feature(macro_metavar_expr_concat)]

use crate::cassette_player::CassettePlayer;

use scamble::expose_dsp;

pub mod cassette_player;

expose_dsp!(CassettePlayer);