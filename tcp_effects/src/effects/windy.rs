use scamble::dsp::{Dsp, DspType};
use scamble::dsp::signal::{Signal, SignalConst, SignalMut};
use rand::{Rng, rng};

pub struct WindySynth {
    tail: f32,
    buf: [f32; 1024],
}

impl Dsp for WindySynth {
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
        WindySynth {
            tail: 0.,
            buf: [0.; 1024],
        }
    }

    fn reset(&mut self) {
        self.tail = 0.;
    }

    fn preferred_out_channels(&self) -> Option<usize> {
        Some(2)
    }

    fn read(&mut self, _: SignalConst, mut output: SignalMut) {
        let ulen = output.length();

        self.buf[0] = shift(self.tail);
        output.write_sample(0, shift(self.tail));
        for i in 1..ulen {
            self.buf[i] = shift(self.buf[i - 1]);
        }

        // low-pass
        for _ in 0..4 {
            self.buf[0] = (self.buf[0] + self.tail) / 2.;
            for i in 1..ulen {
                self.buf[i] = (self.buf[i] + self.buf[i - 1]) / 2.;
            }
        }

        // write output
        for i in 0..ulen {
            output.write_sample(i, self.buf[i]);
        }

        // set tail
        self.tail = self.buf[ulen - 1];
    }
}

fn shift(it: f32) -> f32 {
    (it + rng().random_range(-0.01..0.01)).clamp(-1., 1.)
}
