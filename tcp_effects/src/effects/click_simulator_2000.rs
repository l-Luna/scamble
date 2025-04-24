use scamble::dsp::{Dsp, DspType, ProcessResult};
use scamble::dsp::signal::{Signal, SignalConst, SignalMut};
use circular_buffer::CircularBuffer;
use rand::{Rng, rng};
use realfft::num_complex::Complex;
use realfft::num_traits::Zero;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

// sqrt(2048)
const ADJ: f32 = 45.25483399593904156165403917471;

const BUFLEN: usize = 4096;
const HBUFLEN: usize = BUFLEN / 2 + 1;

pub struct ClickSimulator2000 {
    // previous window of buffered values
    delay_left: CircularBuffer<BUFLEN, f32>,
    delay_right: CircularBuffer<BUFLEN, f32>,
    // FFT instances
    dft_fwd: Arc<dyn RealToComplex<f32>>,
    dft_bwd: Arc<dyn ComplexToReal<f32>>,
    // keep track of previous silence for should-process
    silence: usize,
    // buffers
    scratch: [Complex<f32>; BUFLEN],
    out_left: [Complex<f32>; HBUFLEN],
    out_right: [Complex<f32>; HBUFLEN],
    copy_left: [f32; BUFLEN],
    copy_right: [f32; BUFLEN],
}

impl Dsp for ClickSimulator2000 {
    fn name() -> &'static str {
        "Click Sim"
    }

    fn version() -> u32 {
        1
    }

    fn ty() -> DspType {
        DspType::Effect
    }

    fn create() -> Self {
        let mut planner = RealFftPlanner::new();
        ClickSimulator2000 {
            delay_left: Default::default(),
            delay_right: Default::default(),
            dft_fwd: planner.plan_fft_forward(BUFLEN),
            dft_bwd: planner.plan_fft_inverse(BUFLEN),
            silence: 0,
            scratch: [Complex::zero(); BUFLEN],
            out_left: [Complex::zero(); HBUFLEN],
            out_right: [Complex::zero(); HBUFLEN],
            copy_left: [0.; BUFLEN],
            copy_right: [0.; BUFLEN],
        }
    }

    fn reset(&mut self) {
        self.delay_left.clear();
        self.delay_right.clear();
        self.silence = 0;
    }

    fn should_process(&mut self, idle: bool, incoming_length: usize) -> ProcessResult {
        if idle {
            self.silence += incoming_length;
            if self.silence >= BUFLEN {
                ProcessResult::SkipSilent
            } else {
                ProcessResult::Continue
            }
        } else {
            self.silence = 0;
            ProcessResult::Continue
        }
    }

    fn preferred_out_channels(&self) -> Option<usize> {
        Some(2)
    }

    fn read(&mut self, input: SignalConst, mut output: SignalMut) {
        output.fill(0.);

        // extend buffers
        let (l, r) = input.read_stereo();
        self.delay_left.extend(l);
        self.delay_right.extend(r);

        // when we have enough data...
        if self.delay_left.is_full() && self.delay_right.is_full() {
            // copy from circular buffers to scratch space
            copy_contiguous(&self.delay_left, &mut self.copy_left);
            copy_contiguous(&self.delay_right, &mut self.copy_right);

            // apply forward FFTs
            self.dft_fwd
                .process_with_scratch(&mut self.copy_left, &mut self.out_left, &mut self.scratch)
                .unwrap();
            self.dft_fwd
                .process_with_scratch(&mut self.copy_right, &mut self.out_right, &mut self.scratch)
                .unwrap();

            // processing...
            let adj: f32 = rng().random_range(0.7..1.3);

            // apply filtering
            for i in 0..HBUFLEN {
                self.out_left[i] *= adj;

                // normalize (part 1)
                self.out_left[i] /= ADJ;
            }

            // just in case
            self.out_left[0].im = 0.;
            self.out_left[HBUFLEN - 1].im = 0.;

            // apply backwards FFT
            self.dft_bwd
                .process_with_scratch(&mut self.out_left, &mut self.copy_left, &mut self.scratch)
                .unwrap();

            // normalize outputs (part 2)
            for i in 0..BUFLEN {
                self.copy_left[i] /= ADJ;
            }

            // write to outputs
            let jitter: usize = rng().random_range(0..24);
            for i in 0..output.length() {
                output.write_sample(i, self.copy_left[BUFLEN + i - (output.length() + jitter)]);
            }
        }
    }
}

fn copy_contiguous<const N: usize, T: Copy>(from: &CircularBuffer<N, T>, to: &mut [T]) {
    let (p1, p2) = from.as_slices();
    to[..p1.len()].copy_from_slice(p1);
    to[p1.len()..p1.len() + p2.len()].copy_from_slice(p2);
}

fn lerp(from: f32, to: f32, fact: f32) -> f32 {
    from + (to - from) * fact
}

fn sub_real_cmplx(real: f32, cmplx: Complex<f32>) -> Complex<f32> {
    let (mag, arg) = cmplx.to_polar();
    Complex::from_polar(mag - real, arg)
}

fn interp(x: f32, y: f32, i: usize, l: usize) -> f32 {
    let r = i as f32 / (l - 1) as f32;
    (1. - r) * x + r * y
}
