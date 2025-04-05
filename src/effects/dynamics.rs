use crate::dsp::interop::{Dsp, DspType};
use crate::dsp::signal::{Signal, SignalConst, SignalMut};
use std::simd::num::SimdFloat;
use std::simd::{StdFloat, f32x16};

pub struct LocalDynamics {}

impl Dsp for LocalDynamics {
    fn name() -> &'static str {
        "Local Dynamics"
    }

    fn version() -> u32 {
        1
    }

    fn ty() -> DspType {
        DspType::Effect
    }

    fn create() -> Self {
        LocalDynamics {}
    }

    fn read(&mut self, input: SignalConst, mut output: SignalMut) {
        // vector loop
        let in_data = input.samples();
        let out_data = output.samples_mut();
        let l = in_data.len();
        for i in 0..l / 16 {
            let x = f32x16::load_or(&in_data[i * 16..], f32x16::splat(0.));
            x.abs()
                .sqrt()
                .copysign(x)
                .copy_to_slice(&mut out_data[i * 16..]);
        }
        // leftovers
        for i in l - ((l / 16) * 16)..l {
            let x = in_data[i];
            out_data[i] = x.abs().sqrt().copysign(x);
        }
    }
}
