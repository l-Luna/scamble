/*
use crate::dsp::interop;
use crate::fmod;
use crate::raw_bindings::FMOD_RESULT::FMOD_OK;
use crate::raw_bindings::{FMOD_DEBUG_FLAGS, FMOD_DEBUG_MODE, FMOD_Debug_Initialize, FMOD_RESULT};
use crate::result::FmResultTrait;
use fmod::system::System;
use std::ffi::{CStr, c_char, c_int};
use std::ptr;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn simulate() {
    let system = System::create().fm_unwrap();
    let desc = interop::into_desc::<windy::WindySynth>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let _ = system.play_dsp(&dsp, None, false).fm_unwrap();
    for _ in 0..(7 * 60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap()
}

#[test]
fn sim_effect() {
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
    let desc = interop::into_desc::<click_simulator_2000::ClickSimulator2000>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let sound = system.create_sound("./noisy.mp3").fm_unwrap();
    let channel = system.play_sound(sound, None, true).fm_unwrap();
    channel.add_dsp(0, &dsp).fm_unwrap();
    channel.set_paused(false).fm_unwrap();
    for _ in 0..(80 * 60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap();
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
*/