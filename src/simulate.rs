use std::ffi::{c_char, c_int, CStr};
use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use fmod::system::System;
use crate::{windy, fmod, noise_reduction};
use crate::custom_dsp;
use crate::raw_bindings::{FMOD_Debug_Initialize, FMOD_DEBUG_FLAGS, FMOD_DEBUG_MODE, FMOD_RESULT};
use crate::raw_bindings::FMOD_RESULT::FMOD_OK;
use crate::result::FmResultTrait;

#[test]
fn simulate() {
    let system = System::create().fm_unwrap();
    let desc = custom_dsp::into_desc::<windy::WindySynth>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let channel = system.play_dsp(&dsp, None, false).fm_unwrap();
    for _ in 0..(7*60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap()
}

#[test]
fn sim_effect() {
    unsafe {
        FMOD_Debug_Initialize(0, FMOD_DEBUG_MODE::FMOD_DEBUG_MODE_CALLBACK, Some(fmod_print_callback), ptr::null_mut()).ok_then(|| ()).fm_unwrap();
    }
    let system = System::create().fm_unwrap();
    let desc = custom_dsp::into_desc::<noise_reduction::NoiseReduction>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let sound = system.create_sound("./test_sound_2.mp3").fm_unwrap();
    let channel = system.play_sound(sound, None, true).fm_unwrap();
    channel.add_dsp(0, &dsp).fm_unwrap();
    channel.set_paused(false).fm_unwrap();
    // let channel = system.play_dsp(&dsp, &ChannelGroup::NULL_GROUP, false).fm_unwrap();
    for _ in 0..(50*60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap()
}

extern "C" fn fmod_print_callback(_: FMOD_DEBUG_FLAGS, file: *const c_char, line: c_int, func: *const c_char, msg: *const c_char) -> FMOD_RESULT {
    unsafe {
        let file = CStr::from_ptr(file);
        let func = CStr::from_ptr(func);
        let msg = CStr::from_ptr(msg);
        println!("FMOD: {file:?}::{func:?}@{line}: {msg:?}");
    }
    FMOD_OK
}