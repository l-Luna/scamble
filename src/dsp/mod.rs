use crate::data::*;
use crate::dsp::signal::*;

pub mod decode;
pub mod interop;
pub mod signal;

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
    pub desc: &'static str,
}

pub enum ParameterType<Dsp: ?Sized> {
    Float {
        min: f32,
        max: f32,
        default: f32,
        /* TODO: parameter mapping */
        setter: fn(f32, &mut Dsp),
        getter: fn(&Dsp) -> f32,
    },
    Int {
        min: i32,
        max: i32,
        default: i32,
        max_is_inf: bool,
        names: Option<Vec<&'static str>>,
        setter: fn(i32, &mut Dsp),
        getter: fn(&Dsp) -> i32,
    },
    Bool {
        default: bool,
        names: Option<(&'static str, &'static str)>,
        setter: fn(bool, &mut Dsp),
        getter: fn(&Dsp) -> bool,
    },
    Data {
        setter: fn(&[u8], &mut Dsp),
        getter: fn(&Dsp) -> Option<(&[u8], Option<&str>)>,
    },
    // Max of 1 of each of these:
    /// Provides access to an additional signal input. The parameter itself is set to `true` when
    /// a sidechain input exists; use `interop::with_sidechain` to access the signal itself.
    Sidechain {
        setter: fn(bool, &mut Dsp),
        getter: fn(&Dsp) -> bool,
    },
    DynamicResponse {
        setter: fn(DynamicResponseData, &mut Dsp),
        getter: fn(&Dsp) -> DynamicResponseData,
    },
    /// Read by FMOD Studio to decide when to virtualize sounds.
    OverallGain {
        setter: fn(OverallGainData, &mut Dsp),
        getter: fn(&Dsp) -> OverallGainData,
    },
    /// Set by FMOD Studio with the player's position and attributes.
    ListenerAttributes {
        setter: fn(ListenerAttributesData, &mut Dsp),
        getter: fn(&Dsp) -> ListenerAttributesData,
    },
    /// Set by FMOD Studio with all player's positions and attributes, if there are multiple.
    ListenerAttributesList {
        setter: fn(ListenerAttributesListData, &mut Dsp),
        getter: fn(&Dsp) -> ListenerAttributesListData,
    },
    /// Set by FMOD Studio to the min/max range of the event containing this DSP.
    AttenuationRange {
        setter: fn(AttenuationRangeData, &mut Dsp),
        getter: fn(&Dsp) -> AttenuationRangeData,
    },
    /// Set to provide access to FFT data to games.
    Fft {
        setter: fn(FftData, &mut Dsp),
        getter: fn(&Dsp) -> FftData,
    },
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

    fn reset(&mut self) {}

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

    fn read(&mut self, input: SignalConst, output: SignalMut);
}

impl<T: ?Sized> Parameter<T> {
    pub const fn new(name: &'static str, ty: ParameterType<T>) -> Self {
        Self {
            ty,
            name,
            unit: "",
            desc: "",
        }
    }

    pub const fn with_unit(name: &'static str, unit: &'static str, ty: ParameterType<T>) -> Self {
        Self {
            ty,
            name,
            unit,
            desc: "",
        }
    }
}

// float_param!(note, range: 0.0..1.0, default: 0.0)
#[macro_export]
macro_rules! float_param {
    ($name:ident $(: $t:ty)?, range: $min:literal..$max:literal, default: $default:literal) => {
        ParameterType::Float {
            min: $min,
            max: $max,
            default: $default,
            setter: |value, dsp| dsp.$name = value $(as $t)?,
            getter: |dsp| dsp.$name as f32
        }
    }
}

// int_param!(value: u8, range: 0..24, default: 1)
#[macro_export]
macro_rules! int_param {
    ($name:ident $(: $t:ty)?, range: $min:literal..$max:literal, default: $default:literal) => {
        ParameterType::Int {
            min: $min,
            max: $max,
            default: $default,
            max_is_inf: false,
            names: None,
            setter: |value, dsp| dsp.$name = value $(as $t)?,
            getter: |dsp| dsp.$name as i32
        }
    }
}

// bool_param!(_async, default: false)
#[macro_export]
macro_rules! bool_param {
    ($name:ident, default: $default:literal) => {
        ParameterType::Bool {
            default: $default,
            names: None,
            setter: |value, dsp| dsp.$name = value,
            getter: |dsp| dsp.$name,
        }
    };
}

// enum_param!(voice_mode: VoiceMode, options: [Sum, Average, Solo], default: Solo)
#[macro_export]
macro_rules! enum_param {
    ($name:ident: $t:ty, options: [$($opt:ident $(,)?)*], default: $default:ident) => {
        ParameterType::Int {
            min: 0,
            max: ${count($opt)} - 1,
            default: {
                // aid autocomplete/deref by claiming that it's a variant of $t
                let _: $t = <$t>::$default;
                // ...but match by name in the list
                [$(stringify!($opt),)*].iter().position(|h| *h == stringify!($default)).unwrap() as i32
            },
            max_is_inf: false,
            names: Some(vec![$(stringify!($opt),)*]),
            setter: |value, dsp| dsp.$name = match value {
                $(
                    ${index()} => <$t>::$opt,
                )*
                _ => panic!(concat!("Unknown variant {} for field ", stringify!($name)), value)
            },
            getter: |dsp| match dsp.$name {
                $(
                    <$t>::$opt => ${index()},
                )*
                _ => panic!()
            }
        }
    }
}