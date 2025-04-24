a rust library for writing FMOD DSPs

a simple example:
```rs
#![feature(macro_metavar_expr)]
#![feature(macro_metavar_expr_concat)]

use scamble::expose_dsp;
use scamble::dsp::*;
use scamble::dsp::signal::*;

pub struct Invert;

impl Dsp for Invert{
    fn name() -> &'static str { "Invert" }
    fn version() -> u32 { 1 }
    fn ty() -> DspType { DspType::Effect }
    fn create() -> Self { Invert }
    
    fn read(&mut self, input: SignalConst, mut output: SignalMut) {
        let in_data = input.samples();
        let out_data = output.samples_mut();
        for i in 0..in_data.len() {
            out_data[i] = -in_data[i];
        }
    }
}

expose_dsp!(Invert);
```