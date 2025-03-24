use crate::custom_dsp::{Dsp, DspType, Parameter, ParameterType, ProcessResult};
use circular_buffer::CircularBuffer;
use realfft::num_complex::Complex;
use realfft::num_traits::Zero;
use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use std::sync::Arc;

#[cfg(test)]
use crate::simulate::NRPlot;

// (unused) FFT window
// static FFT_HANN_WINDOW: LazyLock<[f32; 2048]> = LazyLock::new(hann_window);

// sqrt(2048)
const ADJ: f32 = 45.25483399593904156165403917471;

// "how much" low variance signals are reduced
const VAR_ADJ: f32 = 1.6;

// "how quickly" a signal is considered persistent
const PERSIST_FACT: f32 = 0.03;

pub struct NoiseReduction {
    // previous window of buffered values
    delay_left: CircularBuffer<2048, f32>,
    delay_right: CircularBuffer<2048, f32>,
    // accumulated persistent frequencies
    persistent_freqs: [f32; 1025],
    // frequencies over time, for modulation checking
    accumulated_freqs: [CircularBuffer<256, f32>; 16],
    // FFT instances
    dft_fwd: Arc<dyn RealToComplex<f32>>,
    dft_bwd: Arc<dyn ComplexToReal<f32>>,
    dft_mini_fwd: Arc<dyn RealToComplex<f32>>,
    // keep track of previous silence for should-process
    silence: usize,
    // internal clock for plotting
    clock: usize,
    // buffers
    scratch: [Complex<f32>; 2048],
    out_left: [Complex<f32>; 1025],
    out_right: [Complex<f32>; 1025],
    copy_left: [f32; 2048],
    copy_right: [f32; 2048],
    // preserve buffers for plotting during tests
    #[cfg(test)]
    preserve_freqs: [f32; 1024],
    // parameters
    var_adj: f32
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
                    min: 0.1,
                    max: 3.0,
                    default: 1.7,
                    setter: |value, dsp| { dsp.var_adj = value },
                    getter: |x| x.var_adj,
                },
                name: "var_adj",
                unit: "",
                desc: "How much frequency variance is required",
            }
        ]
    }

    fn create() -> Self {
        let mut planner = RealFftPlanner::new();
        NoiseReduction {
            delay_left: Default::default(),
            delay_right: Default::default(),
            persistent_freqs: [0.; 1025],
            accumulated_freqs: Default::default(),
            dft_fwd: planner.plan_fft_forward(2048),
            dft_bwd: planner.plan_fft_inverse(2048),
            dft_mini_fwd: planner.plan_fft_forward(256),
            silence: 0,
            clock: 0,
            scratch: [Complex::zero(); 2048],
            out_left: [Complex::zero(); 1025],
            out_right: [Complex::zero(); 1025],
            copy_left: [0.; 2048],
            copy_right: [0.; 2048],
            #[cfg(test)]
            preserve_freqs: [0.; 1024],

            var_adj: 1.7
        }
    }

    fn reset(&mut self) {
        self.delay_left.clear();
        self.delay_right.clear();
        self.persistent_freqs.fill(0.);
        self.accumulated_freqs.fill(Default::default());
        self.silence = 0;
        self.clock = 0;
    }

    fn should_process(&mut self, idle: bool, incoming_length: usize) -> ProcessResult {
        if idle {
            self.silence += incoming_length;
            if self.silence >= 2048 {
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

    fn read(
        &mut self,
        in_data: &[f32],
        out_data: &mut [f32],
        in_channels: usize, // assume 2
        out_channels: usize,
    ) {
        // extend buffers
        self.delay_left.extend(in_data.iter().step_by(2));
        self.delay_right.extend(in_data.iter().skip(1).step_by(2));

        self.clock += in_data.len() / 2;

        // when we have enough data...
        if self.delay_left.is_full() && self.delay_right.is_full() {
            // copy from circular buffers to scratch space
            copy_contiguous(&self.delay_left, &mut self.copy_left);
            copy_contiguous(&self.delay_right, &mut self.copy_right);

            // calculate max amplitude of recent untouched data for later
            let max_amp = self.copy_left[1024..]
                .iter()
                .map(|x| x.abs())
                .max_by(|x, y| x.total_cmp(y))
                .unwrap();

            // apply window to reduce artefacting (unused)
            /*let window = LazyLock::force(&FFT_HANN_WINDOW);
            for i in 0..2048 {
                self.copy_left[i] *= window[i];
                self.copy_right[i] *= window[i];
            }*/

            // apply forward FFTs
            self.dft_fwd
                .process_with_scratch(&mut self.copy_left, &mut self.out_left, &mut self.scratch)
                .unwrap();
            self.dft_fwd
                .process_with_scratch(&mut self.copy_right, &mut self.out_right, &mut self.scratch)
                .unwrap();

            // hold onto unaltered frequencies for plotting
            #[cfg(test)] {
                for i in 0..1024 {
                    self.preserve_freqs[i] = self.out_left[i].norm().min(self.out_right[i].norm());
                }
            }

            // processing...
            let n = self.out_left.len();

            // find variance of frequencies; noise tends have very low variance compared to speech
            // adjust by first value to keep numeric stability
            let offset = (self.out_left[0] + self.out_right[0]).norm() / 2.;
            let mut mean = 0.;
            let mut variance = 0.;
            for i in 0..n {
                let x = (self.out_left[i] + self.out_right[i]).norm() / 2. - offset;
                mean += x;
                variance += x * x;
            }
            mean /= n as f32;
            variance = ((variance / n as f32) - mean * mean).sqrt();

            // push new accumulated frequency entry
            for i in 0..16 {
                self.accumulated_freqs[i].push_back(0.);
            }

            // apply filtering
            for i in 0..n {
                // take the minimum over each channel for each element
                let left_max = self.out_left[i].norm_sqr() > self.out_right[i].norm_sqr();
                self.out_left[i] = if left_max {
                    self.out_right[i]
                } else {
                    self.out_left[i]
                };

                // update accumulated frequencies
                let norm = self.out_left[i].norm();
                let buffer = &mut self.accumulated_freqs[(i / 64).min(15)];
                let blen = buffer.len() - 1; // lifetime shenanigans
                buffer[blen] += norm;

                // update persistent frequencies
                self.persistent_freqs[i] =
                    lerp(self.persistent_freqs[i].min(norm), norm, PERSIST_FACT);
                // then cut them out
                self.out_left[i] = sub_real_cmplx(self.persistent_freqs[i], self.out_left[i]);

                // normalize (part 1)
                self.out_left[i] /= ADJ;
                // reduce with variance
                let v = self.var_adj;
                self.out_left[i] *= (v - variance.log2().clamp(0., v)) / v;
            }

            #[cfg(test)]
            let mut md = [[0.; 128]; 16];

            // reuse copy_right and out_right for frequency modulation
            for i in 0..16 {
                let mut buf = [0.; 256];
                copy_contiguous(&self.accumulated_freqs[i], &mut buf);
                self.dft_mini_fwd.process_with_scratch(&mut buf, &mut self.out_right[..129], &mut self.scratch).unwrap();
                #[cfg(test)] {
                    for (j, it) in self.out_right[..128].iter().enumerate(){
                        md[i][j] = it.norm();
                    }
                }
            }

            // just in case
            self.out_left[0].im = 0.;
            self.out_left[self.out_left.len() - 1].im = 0.;

            // apply backwards FFT
            self.dft_bwd
                .process_with_scratch(&mut self.out_left, &mut self.copy_left, &mut self.scratch)
                .unwrap();

            // add a minimum value to audio volume
            let adj_amp = max_amp.log10() + 1.8;
            if adj_amp < 0. {
                for i in 0..self.copy_left.len() {
                    self.copy_left[i] /= 2.0 + adj_amp.abs();
                }
            }

            // normalize outputs (part 2)
            let out_len = out_data.len() / out_channels;
            for i in 0..out_len {
                self.copy_left[self.copy_left.len() - out_len + i] /= ADJ;
            }

            // write to outputs
            for i in 0..out_len {
                out_data[i * 2] = self.copy_left[self.copy_left.len() - out_len + i];
                out_data[i * 2 + 1] = self.copy_left[self.copy_left.len() - out_len + i];
            }

            #[cfg(test)] {
                let mut dl = [0.; 2048]; copy_contiguous(&self.delay_left, &mut dl);
                let mut dr = [0.; 2048]; copy_contiguous(&self.delay_right, &mut dr);
                let mut pf = [0.; 1024]; pf.copy_from_slice(&self.persistent_freqs[..1024]);

                crate::simulate::PLOT_QUEUE.push(NRPlot {
                    clock: self.clock,
                    delay_left: dl,
                    delay_right: dr,
                    freqs: self.preserve_freqs,
                    freq_var: variance,
                    persistent_freqs: pf,
                    modulations: md,
                });
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

// (unused) sum a range in the frequency spectrum
/*fn window_sum(slice: &[Complex<f32>], at: usize, radius: usize) -> f32 {
    slice[at.saturating_sub(radius)..(at + radius).min(slice.len())]
        .iter()
        .map(|it| it.norm_sqr())
        .sum()
}*/

// (unused) FFT window
/*const WINDOW_TAPER: usize = 1;
fn hann_window() -> [f32; 2048] {
    let mut arr = [1.; 2048];
    for i in 0..WINDOW_TAPER {
        arr[i] = 0.5 * (1. - f32::cos(2. * f32::PI() * (i as f32) / (WINDOW_TAPER as f32 * 2.)));
    }
    for i in 0..WINDOW_TAPER {
        arr[2047 - i] = 0.5 * (1. - f32::cos(2. * f32::PI() * (i as f32) / (WINDOW_TAPER as f32 * 2.)));
    }
    println!("arr={arr:?}");
    arr
}*/
