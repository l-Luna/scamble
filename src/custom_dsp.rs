use crate::raw_bindings::FMOD_RESULT::{FMOD_ERR_DSP_DONTPROCESS, FMOD_ERR_DSP_SILENCE, FMOD_ERR_PLUGIN};
use crate::raw_bindings::{FMOD_DSP_PARAMETER_DESC__bindgen_ty_1, FMOD_BOOL, FMOD_CHANNELMASK, FMOD_DSP_BUFFER_ARRAY, FMOD_DSP_DESCRIPTION, FMOD_DSP_PARAMETER_DESC, FMOD_DSP_PARAMETER_DESC_BOOL, FMOD_DSP_PARAMETER_DESC_DATA, FMOD_DSP_PARAMETER_DESC_FLOAT, FMOD_DSP_PARAMETER_DESC_INT, FMOD_DSP_PARAMETER_FLOAT_MAPPING, FMOD_DSP_PARAMETER_FLOAT_MAPPING_PIECEWISE_LINEAR, FMOD_DSP_PARAMETER_FLOAT_MAPPING_TYPE, FMOD_DSP_PARAMETER_TYPE, FMOD_DSP_PROCESS_OPERATION, FMOD_DSP_STATE, FMOD_MEMORY_NORMAL, FMOD_PLUGIN_SDK_VERSION, FMOD_RESULT, FMOD_RESULT::FMOD_OK, FMOD_SPEAKERMODE};
use std::ffi::{c_char, c_int, c_uint, c_void, CString};
use std::{panic, ptr};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use std::str::FromStr;

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

pub struct Parameter<Dsp: ?Sized> {
    pub ty: ParameterType<Dsp>,
    pub name: &'static str,
    pub unit: &'static str,
    pub desc: &'static str
}

pub enum ParameterType<Dsp: ?Sized> {
    Float {
        min: f32,
        max: f32,
        default: f32,
        /* TODO: parameter mapping */
        setter: fn(f32, &mut Dsp),
        getter: fn(&Dsp) -> f32
    },
    Int {
        min: i32,
        max: i32,
        default: i32,
        max_is_inf: bool,
        names: Option<Vec<&'static str>>,
        setter: fn(i32, &mut Dsp),
        getter: fn(&Dsp) -> i32
    },
    Bool {
        default: bool,
        names: Option<(&'static str, &'static str)>,
        setter: fn(bool, &mut Dsp),
        getter: fn(&Dsp) -> bool
    },
    Data {
        setter: fn(&[u8], &mut Dsp),
        getter: fn(&Dsp) -> (&[u8], &str)
    },
    // supplied by FMOD Studio
    // TODO: accompanying data structures
    OverallGain,
    _3DAttrs,
    Sidechain,
    Fft,
    _Multi3DAttrs,
    AttenuationRange,
    DynamicResponse
}

pub trait Dsp {
    // branding
    fn name() -> &'static str;
    fn version() -> u32;
    fn ty() -> DspType;

    // parameters
    fn parameters() -> Vec<Parameter<Self>> {
        vec![]
    }

    // lifecycle
    fn create() -> Self;
    fn reset(&mut self);

    // processing
    fn should_process(&mut self, idle: bool, _incoming_length: usize) -> ProcessResult {
        if idle {
            ProcessResult::SkipSilent
        } else {
            ProcessResult::Continue
        }
    }

    fn preferred_out_channels(&self) -> Option<usize> {
        None
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
            use crate::custom_dsp;
            use crate::raw_bindings::FMOD_DSP_DESCRIPTION;
            use core::mem::MaybeUninit;
            use paste::paste;

            paste! {
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
use crate::raw_bindings::FMOD_DSP_PARAMETER_DATA_TYPE::{FMOD_DSP_PARAMETER_DATA_TYPE_3DATTRIBUTES, FMOD_DSP_PARAMETER_DATA_TYPE_3DATTRIBUTES_MULTI, FMOD_DSP_PARAMETER_DATA_TYPE_ATTENUATION_RANGE, FMOD_DSP_PARAMETER_DATA_TYPE_DYNAMIC_RESPONSE, FMOD_DSP_PARAMETER_DATA_TYPE_FFT, FMOD_DSP_PARAMETER_DATA_TYPE_OVERALLGAIN, FMOD_DSP_PARAMETER_DATA_TYPE_SIDECHAIN, FMOD_DSP_PARAMETER_DATA_TYPE_USER};

pub fn into_desc<D: Dsp>() -> FMOD_DSP_DESCRIPTION {
    // name sanitization
    let name = sanitize_str(D::name());
    // buffer counts
    let input_buffers = if D::ty() == DspType::Effect { 1 } else { 0 };
    // parameters
    let params = D::parameters();
    let mut param_conv: Vec<Box<FMOD_DSP_PARAMETER_DESC>> = Vec::new();
    for param in &params {
        let param_desc= CString::from_str(param.desc).unwrap();
        param_conv.push(Box::new(FMOD_DSP_PARAMETER_DESC {
            type_: match &param.ty {
                ParameterType::Float { .. } => FMOD_DSP_PARAMETER_TYPE::FMOD_DSP_PARAMETER_TYPE_FLOAT,
                ParameterType::Int { .. } => FMOD_DSP_PARAMETER_TYPE::FMOD_DSP_PARAMETER_TYPE_INT,
                ParameterType::Bool { .. } => FMOD_DSP_PARAMETER_TYPE::FMOD_DSP_PARAMETER_TYPE_BOOL,
                _ => FMOD_DSP_PARAMETER_TYPE::FMOD_DSP_PARAMETER_TYPE_DATA
            },
            name: sanitize_str(param.name),
            label: sanitize_str(param.unit),
            description: param_desc.into_raw(),
            __bindgen_anon_1: match &param.ty {
                ParameterType::Float { min, max, default, .. }
                    => FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { floatdesc: FMOD_DSP_PARAMETER_DESC_FLOAT {
                        min: *min,
                        max: *max,
                        defaultval: *default,
                        mapping: FMOD_DSP_PARAMETER_FLOAT_MAPPING {
                            type_: FMOD_DSP_PARAMETER_FLOAT_MAPPING_TYPE::FMOD_DSP_PARAMETER_FLOAT_MAPPING_TYPE_LINEAR,
                            piecewiselinearmapping: FMOD_DSP_PARAMETER_FLOAT_MAPPING_PIECEWISE_LINEAR {
                                numpoints: 0,
                                pointparamvalues: ptr::null_mut(),
                                pointpositions: ptr::null_mut(),
                            }
                        }
                    } },
                ParameterType::Int { min, max, default, max_is_inf, names, .. }
                    => FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { intdesc: FMOD_DSP_PARAMETER_DESC_INT {
                    min: *min,
                    max: *max,
                    defaultval: *default,
                    goestoinf: if *max_is_inf { 1 } else { 0 },
                    valuenames: match names {
                        None => ptr::null(),
                        Some(v) => v.iter()
                            .map(|it| CString::from_str(it))
                            .map(|it| it.unwrap().into_raw() as *const c_char)
                            .collect::<Vec<_>>()
                            .leak()
                            .as_ptr()
                    }
                } },
                ParameterType::Bool { names, default, .. }
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { booldesc: FMOD_DSP_PARAMETER_DESC_BOOL {
                        defaultval: if *default { 1 } else { 0 },
                        valuenames: match names {
                            None => ptr::null(),
                            Some((ff, tt)) => {
                                [ff, tt].map(|it| CString::from_str(it))
                                    .map(|it| it.unwrap().into_raw() as *const c_char)
                                    .to_vec()
                                    .leak()
                                    .as_ptr()
                            }
                        },
                    }
                } },
                ParameterType::Data { .. }
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_USER as i32
                    }
                } },
                ParameterType::OverallGain
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_OVERALLGAIN as i32
                    }
                } },
                ParameterType::_3DAttrs
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_3DATTRIBUTES as i32
                    }
                } },
                ParameterType::Sidechain
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_SIDECHAIN as i32
                    }
                } },
                ParameterType::Fft
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_FFT as i32
                    }
                } },
                ParameterType::_Multi3DAttrs
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_3DATTRIBUTES_MULTI as i32
                    }
                } },
                ParameterType::AttenuationRange
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_ATTENUATION_RANGE as i32
                    }
                } },
                ParameterType::DynamicResponse
                    => { FMOD_DSP_PARAMETER_DESC__bindgen_ty_1 { datadesc: FMOD_DSP_PARAMETER_DESC_DATA {
                        datatype: FMOD_DSP_PARAMETER_DATA_TYPE_DYNAMIC_RESPONSE as i32
                    }
                } }
            }
        }));
    }
    FMOD_DSP_DESCRIPTION {
        pluginsdkversion: FMOD_PLUGIN_SDK_VERSION,
        name,
        version: D::version(),
        numinputbuffers: input_buffers,
        numoutputbuffers: 1,
        create: Some(create_callback::<D>),
        release: Some(release_callback::<D>),
        reset: Some(reset_callback::<D>),
        read: /*Some(read_callback::<D>)*/ None,
        process: Some(process_callback::<D>) /*None*/,
        setposition: None,
        numparameters: params.len() as c_int,
        paramdesc: param_conv.leak().as_mut_ptr() as *mut _,
        setparameterfloat: Some(set_param_float_callback::<D>),
        setparameterint: Some(set_param_int_callback::<D>),
        setparameterbool: Some(set_param_bool_callback::<D>),
        setparameterdata: Some(set_param_data_callback::<D>),
        getparameterfloat: Some(get_param_float_callback::<D>),
        getparameterint: Some(get_param_int_callback::<D>),
        getparameterbool: Some(get_param_bool_callback::<D>),
        getparameterdata: Some(get_param_data_callback::<D>),
        shouldiprocess: Some(should_process_callback::<D>),
        userdata: ptr::null_mut(),
        sys_register: None,
        sys_deregister: None,
        sys_mix: None,
    }
}

fn sanitize_str<const N: usize>(mut s: &str) -> [c_char; N]{
    if !s.is_ascii() {
        eprintln!("DSP string {s} is not valid ASCII, replacing with placeholder");
        s = "(Invalid ASCII)";
    }
    if s.len() > N - 1 {
        eprintln!("DSP string {s} is longer than {} characters and will be cut off", N - 1);
        s = &s[0..N - 1];
    }
    let mut san = s.chars().map(|x| x as _).collect::<Vec<_>>();
    san.resize(N, 0);
    san.try_into().unwrap()
}

static DBGSTR: &'static str = "Rust DSP\0";

extern "C" fn create_callback<D: Dsp>(dsp_state: *mut FMOD_DSP_STATE) -> FMOD_RESULT {
    let data = D::create();
    unsafe {
        let mem = (*(*dsp_state).functions).alloc.unwrap()(
            size_of::<D>() as c_uint,
            FMOD_MEMORY_NORMAL,
            DBGSTR.as_ptr() as *const _,
        ) as *mut D;
        ptr::write(mem, data);
        (*dsp_state).plugindata = mem as *mut _;
    }
    FMOD_OK
}

extern "C" fn release_callback<D: Dsp>(dsp_state: *mut FMOD_DSP_STATE) -> FMOD_RESULT {
    unsafe {
        let x = (*dsp_state).plugindata;
        drop(ptr::read(x as *mut D));
        (*(*dsp_state).functions).free.unwrap()(x, FMOD_MEMORY_NORMAL, DBGSTR.as_ptr() as *const _);
        (*dsp_state).plugindata = ptr::null_mut();
    }
    FMOD_OK
}

extern "C" fn reset_callback<D: Dsp>(dsp_state: *mut FMOD_DSP_STATE) -> FMOD_RESULT {
    unsafe {
        let result = panic::catch_unwind(|| {
            let data = &mut *((*dsp_state).plugindata as *mut D);
            data.reset();
        });
        match result{
            Ok(_) => FMOD_OK,
            Err(_) => FMOD_ERR_PLUGIN
        }
    }
}

extern "C" fn should_process_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    idle: FMOD_BOOL,
    length: c_uint,
    _: FMOD_CHANNELMASK, // deprecated
    _: std::os::raw::c_int,
    _: FMOD_SPEAKERMODE,
) -> FMOD_RESULT {
    unsafe {
        let result = panic::catch_unwind(|| {
            let data = &mut *((*dsp_state).plugindata as *mut D);
            match data.should_process(idle != 0, length as usize) {
                ProcessResult::Continue => FMOD_OK,
                ProcessResult::SkipNoEffect => FMOD_ERR_DSP_DONTPROCESS,
                ProcessResult::SkipSilent => FMOD_ERR_DSP_SILENCE,
            }
        });
        match result{
            Ok(_) => FMOD_OK,
            Err(_) => FMOD_ERR_PLUGIN
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
        let result = panic::catch_unwind(|| {
            let data = &mut *((*dsp_state).plugindata as *mut D);
            data.read(
                &*slice_from_raw_parts(in_data, length as usize),
                &mut *slice_from_raw_parts_mut(out_data, length as usize),
                in_channels as usize,
                *out_channels as usize,
            );
        });
        match result{
            Ok(_) => FMOD_OK,
            Err(_) => FMOD_ERR_PLUGIN
        }
    }
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
        let proc = panic::catch_unwind(|| {
            let data = &mut *((*dsp_state).plugindata as *mut D);
            if op == FMOD_DSP_PROCESS_OPERATION::FMOD_DSP_PROCESS_QUERY {
                if let Some(channels) = data.preferred_out_channels() {
                    *(*out_buffers).buffernumchannels = channels as c_int;
                }
                match D::ty() {
                    DspType::Generator => FMOD_OK,
                    DspType::Effect => match data.should_process(idle != 0, length as usize) {
                        ProcessResult::Continue => FMOD_OK,
                        ProcessResult::SkipNoEffect => FMOD_ERR_DSP_DONTPROCESS,
                        ProcessResult::SkipSilent => FMOD_ERR_DSP_SILENCE,
                    },
                }
            } else {
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
        });
        proc.unwrap_or_else(|_| FMOD_ERR_PLUGIN)
    }
}

extern "C" fn set_param_float_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: f32
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Float { setter, .. } = param.ty {
            setter(value, data);
            return FMOD_OK;
        }
    }
    FMOD_ERR_PLUGIN
}

extern "C" fn get_param_float_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: *mut f32,
    _: *mut c_char // valuestr is not currently exposed
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Float { getter, .. } = param.ty {
            unsafe {
                *value = getter(&*data);
                return FMOD_OK;
            }
        }
    }
    FMOD_ERR_PLUGIN
}

extern "C" fn set_param_int_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: i32
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Int { setter, .. } = param.ty {
            setter(value, data);
            return FMOD_OK;
        }
    }
    FMOD_ERR_PLUGIN
}

extern "C" fn get_param_int_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: *mut i32,
    _: *mut c_char // valuestr is not currently exposed
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Int { getter, .. } = param.ty {
            unsafe {
                *value = getter(&*data);
                return FMOD_OK;
            }
        }
    }
    FMOD_ERR_PLUGIN
}

extern "C" fn set_param_bool_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: FMOD_BOOL
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Bool { setter, .. } = param.ty {
            setter(value == 1, data);
            return FMOD_OK;
        }
    }
    FMOD_ERR_PLUGIN
}

extern "C" fn get_param_bool_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: *mut FMOD_BOOL,
    _: *mut c_char // valuestr is not currently exposed
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Bool { getter, .. } = param.ty {
            unsafe {
                *value = if getter(&*data) { 1 } else { 0 };
                return FMOD_OK;
            }
        }
    }
    FMOD_ERR_PLUGIN
}

extern "C" fn set_param_data_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: *mut c_void,
    length: c_uint
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Data { setter, .. } = param.ty {
            unsafe {
                let slc = &*slice_from_raw_parts(value as *mut u8, length as usize);
                setter(slc, data);
            }
            return FMOD_OK;
        }
    }
    FMOD_ERR_PLUGIN
}

extern "C" fn get_param_data_callback<D: Dsp>(
    dsp_state: *mut FMOD_DSP_STATE,
    index: c_int,
    value: *mut *mut c_void,
    length: *mut c_uint,
    desc: *mut c_char
) -> FMOD_RESULT {
    let data = unsafe { &mut *((*dsp_state).plugindata as *mut D) };
    let params = D::parameters();
    if params.len() <= index as usize {
        let param = &params[index as usize];
        if let ParameterType::Data { getter, .. } = param.ty {
            let (c_value, c_desc) = getter(&*data);
            unsafe {
                let target = &mut *slice_from_raw_parts_mut(desc, 32);
                for (i, c) in c_desc[..31].chars().enumerate() {
                    target[i] = c as c_char;
                }
                *value = c_value.as_ptr() as *mut _;
                *length = c_value.len() as c_uint;
                return FMOD_OK;
            }
        }
    }
    FMOD_ERR_PLUGIN
}