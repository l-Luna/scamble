use crate::fmod::dsp::DspInstance;
use crate::raw_bindings::{FMOD_CHANNEL, FMOD_Channel_AddDSP, FMOD_Channel_SetPaused};
use crate::result::FmodResult;
use std::ffi::c_int;

pub struct Channel(pub(in crate::fmod) *mut FMOD_CHANNEL);

impl Channel {
    pub fn set_paused(&self, paused: bool) -> FmodResult<()> {
        unsafe { FMOD_Channel_SetPaused(self.0, if paused { 1 } else { 0 }) }.ok_then(|| ())
    }

    pub fn add_dsp(&self, index: usize, dsp: &DspInstance) -> FmodResult<()> {
        unsafe { FMOD_Channel_AddDSP(self.0, index as c_int, dsp.0) }.ok_then(|| ())
    }
}
