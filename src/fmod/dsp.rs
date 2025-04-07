use crate::data::FftData;
use crate::raw_bindings::*;
use crate::result::FmodResult;
use std::ffi::{c_int, c_uint, c_void};
use std::ptr;
use std::ptr::slice_from_raw_parts;

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

    pub fn get_fft_parameter(&self, index: usize) -> FmodResult<FftData> {
        let mut placement: *mut c_void = ptr::null_mut();
        let mut size_placement: c_uint = 0;
        unsafe {
            FMOD_DSP_GetParameterData(
                self.0,
                index as c_int,
                &mut placement,
                &mut size_placement,
                ptr::null_mut(),
                0,
            )
            .ok_then(|| {
                let it: FMOD_DSP_PARAMETER_FFT = ptr::read(placement as *mut _);
                let mut data = FftData { data: Vec::with_capacity(it.numchannels as usize) };
                for channel_idx in 0..it.numchannels {
                    let spectrum_slice = &*slice_from_raw_parts(
                        it.spectrum[channel_idx as usize],
                        it.length as usize,
                    );
                    data.data.push(Vec::from(spectrum_slice));
                }
                data
            })
        }
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
