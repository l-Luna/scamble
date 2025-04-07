use crate::raw_bindings::*;
use crate::result::FmodResult;
use std::ffi::c_int;
use std::ptr;

#[derive(Copy, Clone)]
pub struct DspInstance(pub(in crate::fmod) *mut FMOD_DSP);

impl DspInstance {
    pub fn get_float_parameter(&self, index: usize) -> FmodResult<f32> {
        let mut result: f32 = 0.;
        unsafe {
            FMOD_DSP_GetParameterFloat(self.0, index as c_int, &mut result, ptr::null_mut(), 0)
        }
        .ok_then(|| result)
    }

    pub fn get_int_parameter(&self, index: usize) -> FmodResult<i32> {
        let mut result: i32 = 0;
        unsafe { FMOD_DSP_GetParameterInt(self.0, index as c_int, &mut result, ptr::null_mut(), 0) }
            .ok_then(|| result)
    }

    pub fn get_bool_parameter(&self, index: usize) -> FmodResult<bool> {
        let mut result: FMOD_BOOL = 0;
        unsafe {
            FMOD_DSP_GetParameterBool(self.0, index as c_int, &mut result, ptr::null_mut(), 0)
        }
        .ok_then(|| result != 0)
    }

    pub fn set_float_parameter(&self, index: usize, value: f32) -> FmodResult<()> {
        unsafe { FMOD_DSP_SetParameterFloat(self.0, index as c_int, value) }.ok_then(|| ())
    }

    pub fn set_bool_parameter(&self, index: usize, value: bool) -> FmodResult<()> {
        unsafe { FMOD_DSP_SetParameterBool(self.0, index as c_int, if value { 1 } else { 0 }) }
            .ok_then(|| ())
    }

    pub fn set_int_parameter(&self, index: usize, value: i32) -> FmodResult<()> {
        unsafe { FMOD_DSP_SetParameterInt(self.0, index as c_int, value) }.ok_then(|| ())
    }
}
