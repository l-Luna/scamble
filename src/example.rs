use crate::custom_dsp::{Dsp, DspType};
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

    fn read(&mut self, _: &[f32], buf: &mut [f32], _: usize, chan: usize) {
        let ulen = buf.len() / chan;

        buf[0] = shift(self.tail);
        for i in 1..ulen {
            buf[i * chan] = shift(buf[(i - 1) * chan]);
        }

        // low-pass
        for _ in 0..4 {
            buf[0] = (buf[0] + self.tail) / 2.;
            for i in 1..ulen {
                buf[i * chan] = (buf[i * chan] + buf[(i - 1) * chan]) / 2.;
            }
        }

        // sync channels
        for i in 0..ulen {
            for j in 1..chan {
                buf[i * chan + j] = buf[i * chan];
            }
        }

        // set tail
        self.tail = buf[ulen * chan - 1];
    }
}

fn shift(it: f32) -> f32 {
    (it + rng().random_range(-0.1..0.1)).clamp(-1., 1.)
}
