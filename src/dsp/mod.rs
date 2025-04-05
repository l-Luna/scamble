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
    // max of 1 of these
    /// Provides access to an additional signal input. The parameter itself is set to `true` when
    /// a sidechain input exists; use `interop::with_sidechain` to access the signal itself.
    Sidechain {
        setter: fn(bool, &mut Dsp),
        getter: fn(&Dsp) -> bool,
    },
    // TODO: accompanying data structures
    DynamicResponse,
    /// Read by FMOD Studio to decide when to virtualize sounds.
    OverallGain,
    /// Set by FMOD Studio with the player's position and attributes.
    ListenerAttributes,
    /// Set by FMOD Studio with all player's positions and attributes, if there are multiple.
    ListenerAttributesList,
    /// Set by FMOD Studio to the min/max range of the event containing this DSP.
    AttenuationRange,
    /// Set to provide access to FFT data to games.
    Fft,
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
