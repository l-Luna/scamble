use crate::dsp::signal::{SignalConst, SignalMut};

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
        getter: fn(&Dsp) -> (&[u8], &str),
    },
    // supplied by FMOD Studio
    // TODO: accompanying data structures
    OverallGain,
    _3DAttrs,
    Sidechain {
        setter: fn(bool, &mut Dsp),
        getter: fn(&Dsp) -> bool,
    },
    Fft,
    _Multi3DAttrs,
    AttenuationRange,
    DynamicResponse,
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
