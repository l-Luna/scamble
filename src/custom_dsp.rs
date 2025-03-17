use std::ffi::c_uint;
use crate::raw_bindings::FMOD_RESULT::{FMOD_ERR_DSP_DONTPROCESS, FMOD_ERR_DSP_SILENCE};
use crate::raw_bindings::{FMOD_BOOL, FMOD_CHANNELMASK, FMOD_DSP_BUFFER_ARRAY, FMOD_DSP_DESCRIPTION, FMOD_DSP_PROCESS_OPERATION, FMOD_DSP_STATE, FMOD_MEMORY_NORMAL, FMOD_PLUGIN_SDK_VERSION, FMOD_RESULT, FMOD_RESULT::FMOD_OK, FMOD_SPEAKERMODE};
use std::{mem, ptr};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DspType {
    Effect,
    Generator,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcessResult {
    Continue,
    SkipNoEffect,
    SkipSilent,
}

pub trait Dsp {
    // branding
    fn name() -> &'static str;
    fn version() -> u32;
    fn ty() -> DspType;

    // lifecycle
    fn create() -> Self;
    fn reset(&mut self);

    // processing
    fn should_process(&self, idle: bool) -> ProcessResult {
        if idle {
            ProcessResult::SkipSilent
        } else {
            ProcessResult::Continue
        }
    }

    fn read(
        &mut self,
        in_data: &[f32],
        out_data: &mut [f32],
        in_channels: usize,
        out_channels: usize,
    );
}

// wrapping DSPs into FMOD's format

#[macro_export]
macro_rules! expose_dsp {
    ($t:ty) => {
        const _: () = {
            use crate::custom_dsp;
            use core::mem::MaybeUninit;
            use crate::raw_bindings::FMOD_DSP_DESCRIPTION;

            static mut DESC: MaybeUninit<FMOD_DSP_DESCRIPTION> = MaybeUninit::uninit();

            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            unsafe extern "stdcall" fn FMODGetDSPDescription() -> *const FMOD_DSP_DESCRIPTION {
                unsafe {
                    (*&raw mut DESC).write(custom_dsp::into_desc::<$t>())
                }
            }
        };
    };
}

pub(crate) use expose_dsp;

pub fn into_desc<D: Dsp>() -> FMOD_DSP_DESCRIPTION {
    // name sanitization
    let mut name = D::name();
    if !name.is_ascii() {
        eprintln!("DSP name {name} is not valid ASCII, replacing with placeholder");
        name = "(Invalid ASCII)";
    }
    if name.len() > 31 {
        eprintln!("DSP name {name} is longer than 31 characters and will be cut off");
        name = &name[0..31];
    }
    let mut san = name.chars().map(|x| x as _).collect::<Vec<_>>();
    san.resize(32, 0);
    // buffer counts
    let input_buffers = if D::ty() == DspType::Effect { 1 } else { 0 };
    //
    FMOD_DSP_DESCRIPTION {
        pluginsdkversion: FMOD_PLUGIN_SDK_VERSION,
        name: san.try_into().unwrap(),
        version: D::version(),
        numinputbuffers: input_buffers,
        numoutputbuffers: 1,
        create: Some(create_callback::<D>),
        release: Some(release_callback::<D>),
        reset: Some(reset_callback::<D>),
        read: /*Some(read_callback::<D>)*/ None,
        process: Some(process_callback::<D>) /*None*/,
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
        shouldiprocess: Some(should_process_callback::<D>),
        userdata: ptr::null_mut(),
        sys_register: None,
        sys_deregister: None,
        sys_mix: None,
    }
}

static dbgstr: &'static str = "Rust DSP\0";

extern "C" fn create_callback<D: Dsp>(dsp_state: *mut FMOD_DSP_STATE) -> FMOD_RESULT {
    let data = D::create();
    unsafe {
        let mem = (*(*dsp_state).functions).alloc.unwrap()(size_of::<D>() as c_uint, FMOD_MEMORY_NORMAL, dbgstr.as_ptr() as *const _) as *mut D;
        ptr::write(mem, data);
        (*dsp_state).plugindata = mem as *mut _;
    }
    FMOD_OK
}

extern "C" fn release_callback<D: Dsp>(dsp_state: *mut FMOD_DSP_STATE) -> FMOD_RESULT {
    unsafe {
        let x = (*dsp_state).plugindata;
        drop(ptr::read(x as *mut D));
        (*(*dsp_state).functions).free.unwrap()(x, FMOD_MEMORY_NORMAL, dbgstr.as_ptr() as *const _);
        (*dsp_state).plugindata = ptr::null_mut();
    }
    FMOD_OK
}

extern "C" fn reset_callback<D: Dsp>(dsp_state: *mut FMOD_DSP_STATE) -> FMOD_RESULT {
    unsafe {
        let data = &mut *((*dsp_state).plugindata as *mut D);
        data.reset();
    }
    FMOD_OK
}

extern "C" fn should_process_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    idle: FMOD_BOOL,
    _: std::os::raw::c_uint,
    _: FMOD_CHANNELMASK, // deprecated
    _: std::os::raw::c_int,
    _: FMOD_SPEAKERMODE,
) -> FMOD_RESULT {
    unsafe {
        let data = &mut *((*dsp_state).plugindata as *mut D);
        match data.should_process(idle == 1) {
            ProcessResult::Continue => FMOD_OK,
            ProcessResult::SkipNoEffect => FMOD_ERR_DSP_DONTPROCESS,
            ProcessResult::SkipSilent => FMOD_ERR_DSP_SILENCE,
        }
    }
}

extern "C" fn read_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    in_data: *mut f32,
    out_data: *mut f32,
    length: std::os::raw::c_uint,
    in_channels: std::os::raw::c_int,
    out_channels: *mut std::os::raw::c_int,
) -> FMOD_RESULT {
    unsafe {
        let data = &mut *((*dsp_state).plugindata as *mut D);
        data.read(
            &*slice_from_raw_parts(in_data, length as usize),
            &mut *slice_from_raw_parts_mut(out_data, length as usize),
            in_channels as usize,
            *out_channels as usize,
        );
    }
    FMOD_OK
}

extern "C" fn process_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    length: std::os::raw::c_uint,
    in_buffers: *const FMOD_DSP_BUFFER_ARRAY,
    out_buffers: *mut FMOD_DSP_BUFFER_ARRAY,
    _: FMOD_BOOL,
    op: FMOD_DSP_PROCESS_OPERATION,
) -> FMOD_RESULT {
    unsafe {
        if op == FMOD_DSP_PROCESS_OPERATION::FMOD_DSP_PROCESS_QUERY {
            *(*out_buffers).buffernumchannels = 2;
            FMOD_OK
        } else {
            let data = &mut *((*dsp_state).plugindata as *mut D);
            let in_chan = (*(*in_buffers).buffernumchannels) as usize;
            let out_chan = (*(*out_buffers).buffernumchannels) as usize;
            data.read(
                &*slice_from_raw_parts(*(*in_buffers).buffers, length as usize * in_chan),
                &mut *slice_from_raw_parts_mut(*(*out_buffers).buffers, length as usize * out_chan),
                in_chan,
                out_chan,
            );
            FMOD_OK
        }
    }
}
