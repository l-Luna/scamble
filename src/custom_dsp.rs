use std::ffi::c_uint;
use crate::raw_bindings::FMOD_RESULT::{FMOD_ERR_DSP_DONTPROCESS, FMOD_ERR_DSP_SILENCE};
use crate::raw_bindings::{FMOD_BOOL, FMOD_CHANNELMASK, FMOD_DSP_BUFFER_ARRAY, FMOD_DSP_DESCRIPTION, FMOD_DSP_PROCESS_OPERATION, FMOD_DSP_STATE, FMOD_MEMORY_NORMAL, FMOD_PLUGIN_SDK_VERSION, FMOD_RESULT, FMOD_RESULT::FMOD_OK, FMOD_SPEAKERMODE};
use std::ptr;
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

// and now the fun stuff

// wrapping DSPs into FMOD's format

#[macro_export]
macro_rules! expose_dsp {
    ($t:ty) => {
        const _: () = {
            use paste::paste;
            use crate::custom_dsp;
            use core::mem::MaybeUninit;
            use crate::raw_bindings::FMOD_DSP_DESCRIPTION;

            paste!{
                #[allow(non_snake_case)]
                static mut [<$t _ DESC>]: MaybeUninit<FMOD_DSP_DESCRIPTION> = MaybeUninit::uninit();
            }

            #[cfg(windows)]
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            #[allow(static_mut_refs)]
            unsafe extern "stdcall" fn FMODGetDSPDescription() -> *const FMOD_DSP_DESCRIPTION {
                unsafe { paste!([<$t _ DESC>]).write(custom_dsp::into_desc::<$t>()) }
            }

            #[cfg(not(windows))]
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            #[allow(static_mut_refs)]
            unsafe extern "C" fn FMODGetDSPDescription() -> *const FMOD_DSP_DESCRIPTION {
                unsafe { paste!([<$t DESC>]).write(custom_dsp::into_desc::<$t>()) }
            }
        };
    };
}

#[macro_export]
macro_rules! expose_dsp_list {
    ($($t:ty $(,)?)*) => {
        const _: () = {
            use core::mem::MaybeUninit;
            use core::ptr;
            use paste::paste;
            use crate::custom_dsp;
            use crate::raw_bindings::FMOD_DSP_DESCRIPTION;
            use crate::raw_bindings::FMOD_PLUGINLIST;
            use crate::raw_bindings::FMOD_PLUGINTYPE::FMOD_PLUGINTYPE_DSP;
            use crate::raw_bindings::FMOD_PLUGINTYPE::FMOD_PLUGINTYPE_MAX;

            $(
                paste!{
                    #[allow(non_upper_case_globals)]
                    static mut [<$t _ DESC>]: MaybeUninit<FMOD_DSP_DESCRIPTION> = MaybeUninit::uninit();

                    #[allow(static_mut_refs)]
                    #[allow(non_snake_case)]
                    fn [<Write $t>]() -> *const FMOD_DSP_DESCRIPTION {
                        unsafe { paste!([<$t _ DESC>]).write(custom_dsp::into_desc::<$t>()) }
                    }
                }
            )*

            static mut PLUGIN_LIST: MaybeUninit<[FMOD_PLUGINLIST; ${count($t)} + 1]> = MaybeUninit::zeroed();

            #[cfg(windows)]
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            #[allow(static_mut_refs)]
            unsafe extern "stdcall" fn FMODGetPluginDescriptionList() -> *const FMOD_PLUGINLIST {
                unsafe {
                    PLUGIN_LIST.write([$( FMOD_PLUGINLIST { type_: FMOD_PLUGINTYPE_DSP, description: paste!([< Write $t >])() as *mut _ }, )* FMOD_PLUGINLIST{ type_: FMOD_PLUGINTYPE_MAX, description: ptr::null_mut() } ]).as_ptr()
                }
            }

            #[cfg(not(windows))]
            #[allow(non_snake_case)]
            #[unsafe(no_mangle)]
            #[allow(static_mut_refs)]
            unsafe extern "C" fn FMODGetPluginDescriptionList() -> *const FMOD_PLUGINLIST {
                unsafe {
                    PLUGIN_LIST.write([$( FMOD_PLUGINLIST { type_: FMOD_PLUGINTYPE_DSP, description: paste!([< Write $t >])() as *mut _ }, )* FMOD_PLUGINLIST{ type_: FMOD_PLUGINTYPE_MAX, description: ptr::null_mut() } ]).as_ptr()
                }
            }
        };
    };
}

pub(crate) use expose_dsp;
pub(crate) use expose_dsp_list;

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
        match data.should_process(idle != 0) {
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
    idle: FMOD_BOOL,
    op: FMOD_DSP_PROCESS_OPERATION,
) -> FMOD_RESULT {
    unsafe {
        if op == FMOD_DSP_PROCESS_OPERATION::FMOD_DSP_PROCESS_QUERY {
            match D::ty() {
                DspType::Generator => {
                    *(*out_buffers).buffernumchannels = 2;
                    FMOD_OK
                }
                DspType::Effect => {
                    let data = &mut *((*dsp_state).plugindata as *mut D);
                    match data.should_process(idle != 0) {
                        ProcessResult::Continue => FMOD_OK,
                        ProcessResult::SkipNoEffect => FMOD_ERR_DSP_DONTPROCESS,
                        ProcessResult::SkipSilent => FMOD_ERR_DSP_SILENCE,
                    }
                }
            }
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
