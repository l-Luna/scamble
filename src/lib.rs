use crate::bindings::FMOD_RESULT::FMOD_OK;
use crate::bindings::{
    FMOD_BOOL, FMOD_CHANNELMASK, FMOD_DSP_BUFFER_ARRAY, FMOD_DSP_DESCRIPTION,
    FMOD_DSP_PROCESS_OPERATION, FMOD_DSP_STATE, FMOD_PLUGIN_SDK_VERSION, FMOD_RESULT,
    FMOD_SPEAKERMODE,
};
use rand::{Rng, rng};
use std::ptr;
use std::slice::from_raw_parts_mut;

pub mod bindings;

static mut DESC: FMOD_DSP_DESCRIPTION = FMOD_DSP_DESCRIPTION {
    pluginsdkversion: FMOD_PLUGIN_SDK_VERSION,
    name: [
        b'S' as i8, b'c' as i8, 'a' as i8, 'm' as i8, 'b' as i8, 'l' as i8, 'e' as i8,
        '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8,
        '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8,
        '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8,
        '\0' as i8, '\0' as i8, '\0' as i8, '\0' as i8,
    ],
    version: 1,
    numinputbuffers: 0,
    numoutputbuffers: 1,
    create: None,
    release: None,
    reset: None,
    read: None,
    process: Some(process),
    setposition: None,
    numparameters: 0,
    paramdesc: ptr::null_mut(),
    setparameterfloat: None,
    setparameterint: None,
    setparameterbool: None,
    setparameterdata: None,
    getparameterfloat: None,
    getparameterint: None,
    getparameterbool: None,
    getparameterdata: None,
    shouldiprocess: Some(should_i_process),
    userdata: ptr::null_mut(),
    sys_register: None,
    sys_deregister: None,
    sys_mix: None,
};

#[unsafe(no_mangle)]
unsafe extern "stdcall" fn FMODGetDSPDescription() -> *mut FMOD_DSP_DESCRIPTION {
    &raw mut DESC
}

extern "C" fn process(
    dsp_state: *mut FMOD_DSP_STATE,
    length: ::std::os::raw::c_uint,
    inbufferarray: *const FMOD_DSP_BUFFER_ARRAY,
    outbufferarray: *mut FMOD_DSP_BUFFER_ARRAY,
    inputsidle: FMOD_BOOL,
    op: FMOD_DSP_PROCESS_OPERATION,
) -> FMOD_RESULT {
    unsafe {
        if op == FMOD_DSP_PROCESS_OPERATION::FMOD_DSP_PROCESS_QUERY {
            *(*outbufferarray).buffernumchannels = 1;
            FMOD_OK
        } else {
            let ulen = length as usize;
            let chann = ((*(*outbufferarray).buffernumchannels) as usize);
            let buf = from_raw_parts_mut(*(*outbufferarray).buffers, ulen * chann);
            for i in 0..ulen * chann {
                buf[i] = rng().random_range(0. ..1.);
            }
            FMOD_OK
        }
    }
}

extern "C" fn should_i_process(
    dsp_state: *mut FMOD_DSP_STATE,
    inputsidle: FMOD_BOOL,
    length: ::std::os::raw::c_uint,
    inmask: FMOD_CHANNELMASK,
    inchannels: ::std::os::raw::c_int,
    speakermode: FMOD_SPEAKERMODE,
) -> FMOD_RESULT {
    FMOD_OK
}
