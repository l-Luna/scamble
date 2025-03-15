use crate::raw_bindings::FMOD_DSP;

pub struct Dsp(pub(in crate::fmod) *mut FMOD_DSP);