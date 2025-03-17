use crate::raw_bindings::FMOD_DSP_DESCRIPTION;

pub mod fmod;
pub mod raw_bindings;

#[cfg(test)]
mod simulate;
mod example;
pub mod custom_dsp;
mod result;

// use crate::custom_dsp;
// use crate::raw_bindings::FMOD_DSP_DESCRIPTION;
use core::mem::MaybeUninit;
use crate::example::DySynth;

static mut DESC: MaybeUninit<FMOD_DSP_DESCRIPTION> = MaybeUninit::zeroed();

#[allow(non_snake_case)]
#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
unsafe extern "stdcall" fn FMODGetDSPDescription() -> *const FMOD_DSP_DESCRIPTION {
    unsafe { DESC.write(custom_dsp::into_desc::<DySynth>()) }
}
