use scamble::dsp::Parameter;
use scamble::dsp::{Dsp, DspType, ParameterType, ProcessResult};
use scamble::dsp::signal::{Signal, SignalConst, SignalMut};
use circular_buffer::CircularBuffer;
use realfft::num_complex::Complex;
use realfft::num_traits::Zero;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::ops::Div;
use std::sync::Arc;

// sqrt(2048)
const ADJ: f32 = 45.25483399593904156165403917471;

const BUFLEN: usize = 4096;
const HBUFLEN: usize = BUFLEN / 2 + 1;

pub struct NoiseReduction {
    // previous window of buffered values
    delay_left: CircularBuffer<BUFLEN, f32>,
    delay_right: CircularBuffer<BUFLEN, f32>,
    // accumulated persistent frequencies
    persistent_freqs: [f32; HBUFLEN],
    // ramp up volume after noise gate is released
    was_gated: bool,
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
    residual: [f32; BUFLEN / 2],
    // parameters
    var_enable: bool,
    var_adj: f32,
    persist_enable: bool,
    persist_lerp: f32,
    noise_gate_enable: bool,
    noise_gate_req: f32,
}

impl Dsp for NoiseReduction {
    fn name() -> &'static str {
        "Noise Reduction"
    }

    fn version() -> u32 {
        1
    }

    fn ty() -> DspType {
        DspType::Effect
    }

    fn parameters() -> Vec<Parameter<Self>> {
        vec![
            Parameter {
                ty: ParameterType::Float {
                    min: 0.,
                    max: 10.,
                    default: 6.,
                    getter: |x| x.var_adj,
                    setter: |value, dsp| dsp.var_adj = value,
                },
                name: "var_adj",
                unit: "",
                desc: "How much frequency variance is required",
            },
            Parameter {
                ty: ParameterType::Bool {
                    default: true,
                    names: None,
                    getter: |x| x.var_enable,
                    setter: |value, dsp| dsp.var_enable = value,
                },
                name: "var_enable",
                unit: "",
                desc: "Whether frequency variance is used",
            },
            Parameter {
                ty: ParameterType::Float {
                    min: 0.01,
                    max: 1.0,
                    default: 0.03,
                    getter: |x| x.persist_lerp,
                    setter: |value, dsp| dsp.persist_lerp = value,
                },
                name: "persist_lerp",
                unit: "per sec",
                desc: "How quickly a frequency is considered persistent",
            },
            Parameter {
                ty: ParameterType::Bool {
                    default: true,
                    names: None,
                    getter: |x| x.persist_enable,
                    setter: |value, dsp| dsp.persist_enable = value,
                },
                name: "persist_enable",
                unit: "",
                desc: "Whether persistent frequencies are analyzed",
            },
            Parameter {
                ty: ParameterType::Bool {
                    default: true,
                    names: None,
                    getter: |x| x.noise_gate_enable,
                    setter: |value, dsp| dsp.noise_gate_enable = value,
                },
                name: "noise_gate_en",
                unit: "",
                desc: "Whether to apply a noise gate",
            },
            Parameter {
                ty: ParameterType::Float {
                    min: -60.,
                    max: 20.,
                    default: -45.,
                    getter: |x| x.noise_gate_req,
                    setter: |value, dsp| dsp.noise_gate_req = value,
                },
                name: "noise_gate_req",
                unit: "dB",
                desc: "The minimum volume for the noise gate",
            },
        ]
    }

    fn create() -> Self {
        let mut planner = RealFftPlanner::new();
        NoiseReduction {
            delay_left: Default::default(),
            delay_right: Default::default(),
            persistent_freqs: [0.; HBUFLEN],
            was_gated: false,
            dft_fwd: planner.plan_fft_forward(BUFLEN),
            dft_bwd: planner.plan_fft_inverse(BUFLEN),
            silence: 0,
            scratch: [Complex::zero(); BUFLEN],
            out_left: [Complex::zero(); HBUFLEN],
            out_right: [Complex::zero(); HBUFLEN],
            copy_left: [0.; BUFLEN],
            copy_right: [0.; BUFLEN],
            residual: [0.; BUFLEN / 2],
            var_enable: true,
            var_adj: 6.,
            persist_enable: true,
            persist_lerp: 0.03,
            noise_gate_enable: true,
            noise_gate_req: -45.,
        }
    }

    fn reset(&mut self) {
        self.delay_left.clear();
        self.delay_right.clear();
        self.persistent_freqs.fill(0.);
        self.was_gated = false;
        self.residual.fill(0.);
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

            // calculate RMS of amplitude to use later
            let rms = if self.noise_gate_enable {
                self.copy_left
                    .iter()
                    .map(|x| x * x)
                    .sum::<f32>()
                    .div(BUFLEN as f32)
                    .sqrt()
            } else {
                0.1
            };

            // apply forward FFTs
            self.dft_fwd
                .process_with_scratch(&mut self.copy_left, &mut self.out_left, &mut self.scratch)
                .unwrap();
            self.dft_fwd
                .process_with_scratch(&mut self.copy_right, &mut self.out_right, &mut self.scratch)
                .unwrap();

            // processing...

            // find variance of frequencies; noise tends have very low variance compared to speech
            // adjust by first value to keep numeric stability
            let mut mean = 0.;
            let mut variance = 0.;
            if self.var_enable {
                let offset = (self.out_left[0].norm() + self.out_right[0].norm()) / 2.;
                for i in 0..HBUFLEN {
                    let x = (self.out_left[i].norm() + self.out_right[i].norm()) / 2. - offset;
                    mean += x;
                    variance += x * x;
                }
                mean /= HBUFLEN as f32;
                variance = ((variance / HBUFLEN as f32) - mean * mean).sqrt();
            }

            // apply filtering
            for i in 0..HBUFLEN {
                // take the minimum over each channel for each element
                let left_max = self.out_left[i].norm_sqr() > self.out_right[i].norm_sqr();
                self.out_left[i] = if left_max {
                    self.out_right[i]
                } else {
                    self.out_left[i]
                };

                // update persistent frequencies
                if self.persist_enable {
                    let norm = self.out_left[i].norm();
                    self.persistent_freqs[i] =
                        lerp(self.persistent_freqs[i].min(norm), norm, self.persist_lerp);
                    // then cut them out
                    self.out_left[i] = sub_real_cmplx(self.persistent_freqs[i], self.out_left[i]);
                } else {
                    self.persistent_freqs[i] = 0.;
                }

                // normalize (part 1)
                self.out_left[i] /= ADJ;

                if self.var_enable {
                    // reduce with variance
                    // self.out_left[i] *= (v - variance.log2().clamp(0., v)) / v;
                    let k = (0.4 * self.var_adj + 6.) / 5.;
                    let x1: f32 = (2. / (1. + f32::powf(2., variance * -k.log2()))) - 1.;
                    self.out_left[i] *= x1;
                }
            }

            // just in case
            self.out_left[0].im = 0.;
            self.out_left[HBUFLEN - 1].im = 0.;

            // apply backwards FFT
            self.dft_bwd
                .process_with_scratch(&mut self.out_left, &mut self.copy_left, &mut self.scratch)
                .unwrap();

            // apply noise gate
            let adj_amp = 20. * rms.log10();
            if self.noise_gate_enable && adj_amp < self.noise_gate_req {
                if self.was_gated {
                    for i in 0..BUFLEN {
                        self.copy_left[i] /= 8.;
                    }
                } else {
                    for i in 0..BUFLEN {
                        self.copy_left[i] /= 1. + 7. * (i as f32 / BUFLEN as f32);
                    }
                }
                self.was_gated = true;
            } else if self.was_gated {
                for i in 0..BUFLEN {
                    self.copy_left[i] /= 1. + 7. * ((BUFLEN - i) as f32 / BUFLEN as f32);
                }
            }

            // normalize outputs (part 2)
            for i in 0..BUFLEN {
                self.copy_left[i] /= ADJ;
            }

            // write to outputs
            let out_len = output.length();
            for i in 0..out_len {
                let new = self.copy_left[BUFLEN - (out_len * 2) + i];
                let old = self.residual[i];
                output.write_sample(i, interp(old, new, i, out_len));
            }

            // update residual
            self.residual[..out_len].copy_from_slice(&self.copy_left[BUFLEN - out_len..]);
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
