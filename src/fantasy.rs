use crate::custom_dsp::{Dsp, DspType, Parameter, ParameterType, ProcessResult};
use circular_buffer::CircularBuffer;
use realfft::num_complex::Complex;
use realfft::num_traits::Zero;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

const BUFLEN: usize = 4096;
const HBUFLEN: usize = BUFLEN / 2 + 1;

pub struct Fantasy {
    delay: CircularBuffer<BUFLEN, f32>,
    silence: usize,

    persistent_freqs: [f32; HBUFLEN],

    scratch: [Complex<f32>; BUFLEN],
    out: [Complex<f32>; HBUFLEN],
    copy: [f32; BUFLEN],
    residual: [f32; BUFLEN / 2],
    dft_fwd: Arc<dyn RealToComplex<f32>>,
    dft_bwd: Arc<dyn ComplexToReal<f32>>,

    detune_window: usize,
    detune_factor: f32,
    detune_bias: bool,

    echo_decay: f32,
    echo_support: f32
}

impl Dsp for Fantasy {
    fn name() -> &'static str {
        "Fantasy"
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
                    min: 0.0,
                    max: 1.0,
                    default: 0.7,
                    setter: |value, dsp| dsp.detune_factor = value,
                    getter: |dsp| dsp.detune_factor,
                },
                name: "detune_factor",
                unit: "",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Int {
                    min: 0,
                    max: 20,
                    default: 8,
                    max_is_inf: false,
                    names: None,
                    setter: |value, dsp| dsp.detune_window = value as usize,
                    getter: |dsp| dsp.detune_window as i32,
                },
                name: "detune_window",
                unit: " bands",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Bool {
                    names: Some(("Forward", "Backwards")),
                    default: true,
                    setter: |value, dsp| dsp.detune_bias = value,
                    getter: |dsp| dsp.detune_bias,
                },
                name: "detune_bias",
                unit: "",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Float {
                    min: 0.5,
                    max: 0.99,
                    default: 0.95,
                    setter: |value, dsp| dsp.echo_decay = value,
                    getter: |dsp| dsp.echo_decay,
                },
                name: "echo_decay",
                unit: "",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Float {
                    min: 0.0,
                    max: 1.0,
                    default: 0.1,
                    setter: |value, dsp| dsp.echo_support = value,
                    getter: |dsp| dsp.echo_support,
                },
                name: "echo_support",
                unit: "",
                desc: "",
            }
        ]
    }

    fn create() -> Self {
        let mut planner = RealFftPlanner::new();
        Fantasy {
            delay: Default::default(),
            silence: 0,

            persistent_freqs: [0.; HBUFLEN],

            scratch: [Complex::zero(); BUFLEN],
            out: [Complex::zero(); HBUFLEN],
            copy: [0.; BUFLEN],
            residual: [0.; BUFLEN / 2],
            dft_fwd: planner.plan_fft_forward(BUFLEN),
            dft_bwd: planner.plan_fft_inverse(BUFLEN),

            detune_factor: 0.7,
            detune_window: 8,
            detune_bias: true,

            echo_decay: 0.95,
            echo_support: 0.1
        }
    }

    fn reset(&mut self) {
        self.delay.fill(0.);
        self.silence = 0;
        self.persistent_freqs.fill(0.);
        self.residual.fill(0.);
    }

    fn should_process(&mut self, idle: bool, incoming_length: usize) -> ProcessResult {
        if idle {
            self.silence += incoming_length;
            if self.silence >= BUFLEN && self.persistent_freqs.iter().all(|it| *it <= 0.0001) {
                ProcessResult::SkipSilent
            } else {
                ProcessResult::Continue
            }
        } else {
            self.silence = 0;
            ProcessResult::Continue
        }
    }

    fn read(
        &mut self,
        in_data: &[f32],
        out_data: &mut [f32],
        in_channels: usize,
        out_channels: usize,
    ) {
        out_data.fill(0.);

        self.delay.extend(
            in_data
                .iter()
                .step_by(2)
                .zip(in_data.iter().skip(1).step_by(2))
                .map(|(l, r)| (l + r) / 2.),
        );

        if self.delay.is_full() {
            copy_contiguous(&self.delay, &mut self.copy);
            self.dft_fwd
                .process_with_scratch(&mut self.copy, &mut self.out, &mut self.scratch)
                .unwrap();

            self.scratch[..HBUFLEN].copy_from_slice(&self.out);
            for i in 8..HBUFLEN-8 {
                let orig_norm = self.out[i].norm();
                let mut target_norm = orig_norm;

                let window: usize = self.detune_window;
                if window > 0 {
                    let start = window.saturating_sub(i);
                    let end = (2 * window + 1).min(HBUFLEN - 2 - i);
                    let range = start..=end;
                    if self.detune_bias {
                        for j in range { // the world if you could `let it: impl Iterator<...>`
                            target_norm = (1. - self.detune_factor) * target_norm + self.detune_factor * self.scratch[i + j - window].norm();
                        }
                    } else {
                        for j in range.rev() {
                            target_norm = (1. - self.detune_factor) * target_norm + self.detune_factor * self.scratch[i + j - window].norm();
                        }
                    }
                    //target_norm = (i-range..=i+range).map(|i| self.scratch[i].norm()).sum::<f32>() / (range * 2 + 1) as f32;
                }

                target_norm += self.persistent_freqs[i];

                if orig_norm < 0.000001 {
                    //self.out[i] = Complex::from(target_norm);
                } else {
                    self.out[i] *= target_norm / orig_norm;
                }

                self.persistent_freqs[i] += orig_norm * self.echo_support;
                self.persistent_freqs[i] *= self.echo_decay;
            }

            self.dft_bwd
                .process_with_scratch(&mut self.out, &mut self.copy, &mut self.scratch)
                .unwrap();
        }

        let out_len = out_data.len() / out_channels;
        for i in 0..BUFLEN {
            self.copy[i] /= BUFLEN as f32;
        }

        // write to outputs
        for i in 0..out_len {
            let new = self.copy[BUFLEN - (out_len * 2) + i];
            let old = self.residual[i];
            let it = interp(old, new, i, out_len);
            out_data[i * 2] = it;
            out_data[i * 2 + 1] = it;
        }

        // update residual
        self.residual[..out_len].copy_from_slice(&self.copy[BUFLEN - out_len..]);
    }
}

fn copy_contiguous<const N: usize, T: Copy>(from: &CircularBuffer<N, T>, to: &mut [T]) {
    let (p1, p2) = from.as_slices();
    to[..p1.len()].copy_from_slice(p1);
    to[p1.len()..p1.len() + p2.len()].copy_from_slice(p2);
}

fn interp(x: f32, y: f32, i: usize, l: usize) -> f32 {
    let r = i as f32 / (l - 1) as f32;
    (1. - r) * x + r * y
}