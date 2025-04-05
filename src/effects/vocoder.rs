use crate::dsp::Parameter;
use crate::dsp::interop::{Dsp, DspType, ParameterType, ProcessResult, with_sidechain};
use crate::dsp::signal::{Signal, SignalConst, SignalMut};
use circular_buffer::CircularBuffer;
use realfft::num_complex::Complex;
use realfft::num_traits::Zero;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

const BUFLEN: usize = 4096;
const HBUFLEN: usize = BUFLEN / 2 + 1;

pub struct Vocoder {
    delay_carrier: CircularBuffer<BUFLEN, f32>,
    delay_signal: CircularBuffer<BUFLEN, f32>,
    silence: usize,

    scratch: [Complex<f32>; BUFLEN],
    out_carrier: [Complex<f32>; HBUFLEN],
    out_signal: [Complex<f32>; HBUFLEN],
    copy: [f32; BUFLEN],
    residual: [f32; BUFLEN / 2],
    dft_fwd: Arc<dyn RealToComplex<f32>>,
    dft_bwd: Arc<dyn ComplexToReal<f32>>,

    sidechain_enabled: bool,

    carrier_offset: i32,
    signal_offset: i32,
}

impl Dsp for Vocoder {
    fn name() -> &'static str {
        "Vocoder"
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
                ty: ParameterType::Sidechain {
                    setter: |value, dsp| dsp.sidechain_enabled = value,
                    getter: |dsp| dsp.sidechain_enabled,
                },
                name: "sidechain",
                unit: "",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Int {
                    min: -80,
                    max: 80,
                    default: 0,
                    max_is_inf: false,
                    names: None,
                    setter: |value, dsp| dsp.carrier_offset = value,
                    getter: |dsp| dsp.carrier_offset,
                },
                name: "carrier_offset",
                unit: "bands",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Int {
                    min: -80,
                    max: 80,
                    default: 0,
                    max_is_inf: false,
                    names: None,
                    setter: |value, dsp| dsp.signal_offset = value,
                    getter: |dsp| dsp.signal_offset,
                },
                name: "signal_offset",
                unit: "bands",
                desc: "",
            },
        ]
    }

    fn create() -> Self {
        let mut planner = RealFftPlanner::new();
        Vocoder {
            delay_carrier: Default::default(),
            delay_signal: Default::default(),
            silence: 0,
            scratch: [Complex::zero(); BUFLEN],
            out_carrier: [Complex::zero(); HBUFLEN],
            out_signal: [Complex::zero(); HBUFLEN],
            copy: [0.; BUFLEN],
            residual: [0.; BUFLEN / 2],
            dft_fwd: planner.plan_fft_forward(BUFLEN),
            dft_bwd: planner.plan_fft_inverse(BUFLEN),
            sidechain_enabled: false,
            carrier_offset: 0,
            signal_offset: 0,
        }
    }

    fn reset(&mut self) {
        self.delay_carrier.fill(0.);
        self.delay_signal.fill(0.);
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

    fn read(&mut self, input: SignalConst, mut output: SignalMut) {
        output.fill(0.);

        // extend buffers
        self.delay_signal.extend(input.read_mono());

        if self.sidechain_enabled {
            with_sidechain(|rst| {
                if let Some(carrier) = rst {
                    self.delay_carrier.extend(carrier.read_mono());

                    if self.delay_signal.is_full() && self.delay_carrier.is_full() {
                        copy_contiguous(&self.delay_signal, &mut self.copy);
                        self.dft_fwd
                            .process_with_scratch(
                                &mut self.copy,
                                &mut self.out_signal,
                                &mut self.scratch,
                            )
                            .unwrap();

                        copy_contiguous(&self.delay_carrier, &mut self.copy);
                        self.dft_fwd
                            .process_with_scratch(
                                &mut self.copy,
                                &mut self.out_carrier,
                                &mut self.scratch,
                            )
                            .unwrap();

                        let max_carrier = self
                            .out_carrier
                            .iter()
                            .max_by(|l, r| l.norm_sqr().total_cmp(&r.norm_sqr()))
                            .unwrap()
                            .norm();

                        for i in 0..HBUFLEN {
                            self.out_carrier[i] /= max_carrier;
                        }

                        self.scratch[..HBUFLEN].copy_from_slice(&self.out_signal);

                        for i in 0..HBUFLEN {
                            let sig_i = i as i32 + self.signal_offset;
                            let car_i = i as i32 + self.carrier_offset;
                            if sig_i < 0
                                || sig_i >= HBUFLEN as i32
                                || car_i < 0
                                || car_i >= HBUFLEN as i32
                            {
                                self.out_signal[i] = Complex::zero();
                            } else {
                                self.out_signal[i] = self.scratch[sig_i as usize]
                                    * self.out_carrier[car_i as usize].norm();
                            }
                        }
                        self.out_signal[0].im = 0.;
                        self.out_signal[HBUFLEN - 1].im = 0.;

                        self.dft_bwd
                            .process_with_scratch(
                                &mut self.out_signal,
                                &mut self.copy,
                                &mut self.scratch,
                            )
                            .unwrap();

                        let out_len = output.length();
                        for i in 0..BUFLEN {
                            self.copy[i] /= BUFLEN as f32;
                        }

                        // write to outputs
                        for i in 0..out_len {
                            let new = self.copy[BUFLEN - (out_len * 2) + i];
                            let old = self.residual[i];
                            output.write_sample(i, interp(old, new, i, out_len));
                        }

                        // update residual
                        self.residual[..out_len].copy_from_slice(&self.copy[BUFLEN - out_len..]);
                    }
                }
            });
        } else {
            self.delay_carrier.fill(0.);
        }
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
