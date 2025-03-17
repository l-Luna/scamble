use crate::fmod::custom_dsp::{Dsp, DspType};
use rand::{Rng, rng};

pub struct DySynth {
    tail: f32,
}

impl Dsp for DySynth {
    fn name() -> &'static str {
        "Scamble"
    }

    fn version() -> u32 {
        1
    }

    fn ty() -> DspType {
        DspType::Generator
    }

    fn create() -> Self {
        DySynth { tail: 0. }
    }

    fn reset(&mut self) {
        self.tail = 0.;
    }

    fn read(&mut self, _: &[f32], buf: &mut [f32], _: usize, out_chan: usize) {
        let ulen = buf.len() / out_chan;

        buf[0] = shift(self.tail);
        for i in 1..ulen {
            buf[i * 2] = shift(buf[i * 2 - 2]);
        }

        // low-pass
        for _ in 0..4 {
            for i in 1..ulen {
                buf[i * 2] = (buf[i * 2] + buf[i * 2 - 2]) / 2.;
            }
        }

        // sync buffers
        for i in 0..ulen {
            buf[i * 2 + 1] = buf[i * 2];
        }

        // set tail
        self.tail = buf[ulen * 2 - 1];
    }
}

fn shift(it: f32) -> f32 {
    (it + rng().random_range(-0.05..0.05)).clamp(-1., 1.)
}
