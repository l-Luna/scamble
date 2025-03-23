use crate::custom_dsp;
use crate::raw_bindings::FMOD_RESULT::FMOD_OK;
use crate::raw_bindings::{FMOD_DEBUG_FLAGS, FMOD_DEBUG_MODE, FMOD_Debug_Initialize, FMOD_RESULT};
use crate::result::FmResultTrait;
use crate::{fmod, noise_reduction, windy};
use fmod::system::System;
use lockfree::queue::Queue;
use plotters::prelude::full_palette::GREY;
use plotters::prelude::*;
use std::ffi::{CStr, c_char, c_int};
use std::ptr;
use std::sync::LazyLock;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn simulate() {
    let system = System::create().fm_unwrap();
    let desc = custom_dsp::into_desc::<windy::WindySynth>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let channel = system.play_dsp(&dsp, None, false).fm_unwrap();
    for _ in 0..(7 * 60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap()
}

#[cfg(test)]
#[derive(Debug)]
pub struct NRPlot {
    pub clock: usize,
    pub delay_left: [f32; 2048],
    pub delay_right: [f32; 2048],
    pub freqs: [f32; 1024],
    pub freq_var: f32,
    pub persistent_freqs: [f32; 1024],
    pub modulations: [[f32; 128]; 16],
}

#[cfg(test)]
pub static PLOT_QUEUE: LazyLock<Queue<NRPlot>> = LazyLock::new(|| Queue::new());

#[test]
fn sim_effect() {
    LazyLock::force(&PLOT_QUEUE);

    unsafe {
        FMOD_Debug_Initialize(
            0,
            FMOD_DEBUG_MODE::FMOD_DEBUG_MODE_CALLBACK,
            Some(fmod_print_callback),
            ptr::null_mut(),
        )
        .ok_then(|| ())
        .fm_unwrap();
    }
    let system = System::create().fm_unwrap();
    let desc = custom_dsp::into_desc::<noise_reduction::NoiseReduction>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let sound = system.create_sound("./noisy.mp3").fm_unwrap();
    let channel = system.play_sound(sound, None, true).fm_unwrap();
    channel.add_dsp(0, &dsp).fm_unwrap();
    channel.set_paused(false).fm_unwrap();
    for _ in 0..(25 * 60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap();

    // plot the data
    let root = BitMapBackend::new("plot.png", (1920, 1080)).into_drawing_area();

    let parts = root.split_evenly((5, 1));
    for part in &parts {
        part.fill(&WHITE).unwrap();
    }

    let length = 800000.;
    let mut samples_chart = ChartBuilder::on(&parts[0])
        .build_cartesian_2d(2048. ..length, -1. ..1.)
        .unwrap();
    let mut spectrogram_chart = ChartBuilder::on(&parts[1])
        .build_cartesian_2d(2048. ..length, 0. ..1024.)
        .unwrap();
    let mut variances_chart = ChartBuilder::on(&parts[2])
        .build_cartesian_2d(2048. ..length, 0. ..10.)
        .unwrap();
    let mut variances = Vec::new();
    let mut pers_spectrogram_chart = ChartBuilder::on(&parts[3])
        .build_cartesian_2d(2048. ..length, 0. ..1024.)
        .unwrap();
    let mut modulations_chart = ChartBuilder::on(&parts[4])
        .build_cartesian_2d(2048. ..length, 0. ..16.)
        .unwrap();

    let mut last_clock = 0;
    for sample in PLOT_QUEUE.pop_iter() {
        let clock = sample.clock;

        samples_chart
            .draw_series(LineSeries::new(
                sample.delay_right[1024..]
                    .iter()
                    .enumerate()
                    .map(|(i, s)| ((clock + i) as f64, *s as f64)),
                &GREY,
            ))
            .unwrap();
        samples_chart
            .draw_series(LineSeries::new(
                sample.delay_left[1024..]
                    .iter()
                    .enumerate()
                    .map(|(i, s)| ((clock + i) as f64, *s as f64)),
                &BLACK,
            ))
            .unwrap();

        spectrogram_chart
            .draw_series(sample.freqs.iter().enumerate().map(|(i, amp)| {
                let rect = Rectangle::new(
                    [
                        (last_clock as f64, i as f64),
                        (clock as f64 + 1., i as f64 + 1.),
                    ],
                    Bone.get_color_normalized((*amp as f64 + 1.).log2(), 0., 1.6)
                        .filled(),
                );
                rect
            }))
            .unwrap();

        pers_spectrogram_chart
            .draw_series(sample.persistent_freqs.iter().enumerate().map(|(i, amp)| {
                let rect = Rectangle::new(
                    [
                        (last_clock as f64, i as f64),
                        (clock as f64 + 1., i as f64 + 1.),
                    ],
                    Bone.get_color_normalized((*amp as f64 + 1.).log2(), 0., 1.6)
                        .filled(),
                );
                rect
            }))
            .unwrap();

        modulations_chart
            .draw_series(sample.modulations.iter().enumerate().map(|(i, amp)| {
                let strength: f32 = amp.iter().sum();
                // average location of values in data, 0..1
                let mut directivity: f32 = amp
                    .iter()
                    .enumerate()
                    .map(|(idx, value)| idx as f32 * *value)
                    .sum::<f32>()
                    / (strength * 64.);
                if strength == 0. {
                    directivity = 0.;
                }
                let reduced_str = ((strength as f64 + 1.).log(75.) - 1.75) as f32;
                let col = RGBColor(
                    ((1. - directivity) * reduced_str * 255.) as u8,
                    0,
                    (directivity * reduced_str * 255.) as u8,
                );
                let rect = Rectangle::new(
                    [
                        (last_clock as f64, i as f64),
                        (clock as f64 + 1., i as f64 + 1.),
                    ],
                    col.filled(),
                );
                rect
            }))
            .unwrap();

        variances.push((clock, sample.freq_var));

        last_clock = clock;
    }

    variances_chart
        .draw_series(LineSeries::new(
            variances
                .iter()
                .enumerate()
                .map(|(i, (clock, variance))| ((clock + i) as f64, *variance as f64)),
            &BLACK,
        ))
        .unwrap();

    samples_chart
        .configure_mesh()
        .max_light_lines(4)
        .draw()
        .unwrap();
    variances_chart
        .configure_mesh()
        .max_light_lines(4)
        .draw()
        .unwrap();
}

extern "C" fn fmod_print_callback(
    _: FMOD_DEBUG_FLAGS,
    file: *const c_char,
    line: c_int,
    func: *const c_char,
    msg: *const c_char,
) -> FMOD_RESULT {
    unsafe {
        let file = CStr::from_ptr(file);
        let func = CStr::from_ptr(func);
        let msg = CStr::from_ptr(msg);
        println!("FMOD: {file:?}::{func:?}@{line}: {msg:?}");
    }
    FMOD_OK
}
