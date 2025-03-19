use crate::custom_dsp::{Dsp, DspType};
use std::simd::{StdFloat, f32x64};

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

    fn reset(&mut self) {}

    fn read(&mut self, in_data: &[f32], out_data: &mut [f32], _: usize, _: usize) {
        // vector loop
        let l = in_data.len();
        for i in 0..l / 64 {
            f32x64::load_or(&in_data[i * 64..], f32x64::splat(0.))
                .sin()
                .copy_to_slice(&mut out_data[i * 64..]);
        }
        // leftovers
        for i in l - (l / 64) * 64..l {
            let x = in_data[i];
            out_data[i] = x.abs().sqrt().copysign(x);
        }
    }
}
