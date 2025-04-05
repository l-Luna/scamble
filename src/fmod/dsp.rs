use crate::raw_bindings::FMOD_DSP;

pub struct DspInstance(pub(in crate::fmod) *mut FMOD_DSP);
