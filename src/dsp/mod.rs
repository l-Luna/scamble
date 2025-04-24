//! Tools for creating DSPs.

use crate::data::*;
use crate::dsp::signal::*;

pub mod decode;
pub mod interop;
pub mod signal;

/// A DSP type, used to distinguish effects from generators/instruments.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DspType {
    /// An effect DSP that processes input. These appear in FMOD Studio as plug-in effects.
    Effect,
    /// A generator DSP that produces output without an input. These appear in FMOD Studio as plug-in
    /// instruments.
    ///
    /// Generators must override [Dsp::preferred_out_channels] to produce any output.
    Generator,
}

/// An outcome of processing a DSP on a particular input and state.
/// FMOD uses this value to determine whether to skip processing, and when to end an event entirely.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcessResult {
    /// Processing this DSP will produce audible output, so it should run.
    Continue,
    /// Processing this DSP will have no effect, which should be passed through instead.
    SkipNoEffect,
    /// Processing this DSP will produce silence, so it should be skipped.
    SkipSilent,
}

/// A parameter of a DSP.
pub struct Parameter<Dsp: ?Sized> {
    /// The parameter's type, and parameter-type specific details, including getters, setters, and ranges.
    pub ty: ParameterType<Dsp>,
    /// The parameter's name.
    pub name: &'static str,
    /// The parameter's unit. FMOD Studio provides preferential treatment for the following units:
    /// - Hz
    /// - ms
    /// - st
    /// - dB
    /// - %
    /// - Deg
    pub unit: &'static str,
    /// The parameter's description. Unused by FMOD Studio.
    pub desc: &'static str,
}

/// A parameter's type, and type-specific details.
pub enum ParameterType<Dsp: ?Sized> {
    /// An automatable float parameter.
    Float {
        /// The minimum value.
        min: f32,
        /// The maximum value.
        max: f32,
        /// The default value.
        default: f32,
        /* TODO: parameter mapping */
        /// Callback used to set this parameter.
        setter: fn(f32, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> f32,
    },
    /// A non-automatable, signed 32-bit integer parameter. Can be presented and used as an enumeration
    /// or dropdown.
    Int {
        /// The minimum value.
        min: i32,
        /// The maximum value
        max: i32,
        /// The default value.
        default: i32,
        /// Whether the maximum value should be displayed as "infinity" in UIs.
        max_is_inf: bool,
        /// String names associated with values of this parameter, used to present an integer parameter
        /// as an enumeration or dropdown.
        names: Option<Vec<&'static str>>,
        /// Callback used to set this parameter.
        setter: fn(i32, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> i32,
    },
    /// A non-automatable boolean parameter.
    Bool {
        /// The default value.
        default: bool,
        /// String names associated with `true` and `false`.
        names: Option<(&'static str, &'static str)>,
        /// Callback used to set this parameter.
        setter: fn(bool, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> bool,
    },
    /// A binary data parameter.
    ///
    /// Note that all builtin FMOD data parameter types are presented as different types.
    Data {
        /// Callback used to set this parameter.
        setter: fn(&[u8], &mut Dsp),
        /// Callback used to get this parameter. Returning [None] has the effect of returning
        /// a zero-sized buffer. Optionally, a string description may also be returned.
        getter: fn(&Dsp) -> Option<(&[u8], Option<&str>)>,
    },
    // Max of 1 of each of these:
    /// A sidechain parameter. At most one may be present.
    ///
    /// Provides access to an additional signal input. The parameter itself is set to `true` when
    /// a sidechain input exists; use `interop::with_sidechain` to access the signal itself.
    Sidechain {
        /// Callback used to set the parameter's "enable" flag.
        setter: fn(bool, &mut Dsp),
        /// Callback used to get the parameter's "enable" flag.
        getter: fn(&Dsp) -> bool,
    },
    /// A dynamic response parameter. At most one may be present. Programs may get this for analysis;
    /// this is not presented in UIs.
    DynamicResponse {
        /// Callback used to set this parameter.
        setter: fn(DynamicResponseData, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> DynamicResponseData,
    },
    /// An overall gain parameter. At most one may be present. Read by FMOD Studio to decide when to
    /// virtualize sounds; this is not presented in UIs.
    OverallGain {
        /// Callback used to set this parameter.
        setter: fn(OverallGainData, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> OverallGainData,
    },
    /// A listener attributes parameter. At most one may be present. Set by FMOD Studio with the
    /// listener's position and attributes; this is not presented in UIs.
    ListenerAttributes {
        /// Callback used to set this parameter.
        setter: fn(ListenerAttributesData, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> ListenerAttributesData,
    },
    /// A listener attributes list parameter. At most one may be present. Set by FMOD Studio with
    /// all listener's positions and attributes, if there are multiple; this is not presented in UIs.
    ListenerAttributesList {
        /// Callback used to set this parameter.
        setter: fn(ListenerAttributesListData, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> ListenerAttributesListData,
    },
    /// An attenuation range parameter. At most one may be present. Set by FMOD Studio to the
    /// minimum/maximum range of the event containing this DSP.
    ///
    /// This is not presented in UIs directly, but is affected by the event-wide options. You may
    /// provide your own parameters and logic to allow users to override the values set here.
    AttenuationRange {
        /// Callback used to set this parameter.
        setter: fn(AttenuationRangeData, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> AttenuationRangeData,
    },
    /// A frequency spectrum parameter. At most one may be present. Set to provide access to spectrum
    /// data to programs; this is neither read, written, or displayed by FMOD.
    Fft {
        /// Callback used to set this parameter.
        setter: fn(FftData, &mut Dsp),
        /// Callback used to get this parameter.
        getter: fn(&Dsp) -> FftData,
    },
}

/// A DSP, either an effect or generator. Any type that implements this trait can be converted to
/// an FMOD [crate::raw_bindings::FMOD_DSP_DESCRIPTION] using [interop::into_desc].
pub trait Dsp {
    // branding
    /// Gets the DSP's name. Must be up to 31 characters, though a plugin script can provide a longer
    /// user-facing one. Bank files and scripts refer to names, so changing them is a breaking change.
    fn name() -> &'static str;
    /// Gets the DSP's version. Should be changed every time the DSP changes.
    fn version() -> u32;
    /// Gets the type of the DSP, either an effect or generator (plug-in instrument.)
    fn ty() -> DspType;

    // parameters
    /// Gets the DSP's parameters.
    fn parameters() -> Vec<Parameter<Self>> {
        vec![]
    }

    // lifecycle
    /// Creates an instance of the DSP with default settings.
    fn create() -> Self;

    /// Resets the DSP's internal state.
    fn reset(&mut self) {}

    // processing
    /// Checks whether the DSP will produce any input for the given input and internal state.
    /// `idle` is `true` when the input is silent. The default implementation proceeds with
    /// processing whenever the input is non-silent.
    ///
    /// FMOD uses the returned value for both optimization and to decide when to end an event,
    /// and returning an incorrect value can both prolong an event indefinitely and cut off a sound
    /// prematurely.
    fn should_process(&mut self, idle: bool, _incoming_length: usize) -> ProcessResult {
        if idle {
            ProcessResult::SkipSilent
        } else {
            ProcessResult::Continue
        }
    }

    /// Gets the number of output channels of the DSP, or [None] if the DSP doesn't alter channel
    /// count.
    ///
    /// Generator DSPs must override this to produce output. Other DSPs that explicitly affect
    /// channel routing (e.g. panning, spatializing) should do so based on state; otherwise, prefer
    /// to be generic over channel count.
    fn preferred_out_channels(&self) -> Option<usize> {
        None
    }

    /// Process a block of input into a block of output through this DSP.
    ///
    /// The output must be fully written to by the DSP.
    fn read(&mut self, input: SignalConst, output: SignalMut);
}

impl<T: ?Sized> Parameter<T> {
    /// Create a new instance of a parameter, with the given name and type-specific data.
    pub const fn new(name: &'static str, ty: ParameterType<T>) -> Self {
        Self {
            ty,
            name,
            unit: "",
            desc: "",
        }
    }

    /// Create a new instance of a parameter, with the given name, unit, and type-specific data.
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
/// Convenience macro for quickly creating a float parameter based on a field.
///
/// Example:
/// ```
/// use scamble::dsp::{Parameter, ParameterType};
/// use scamble::float_param;
///
/// struct MyDsp {
///     intensity: f32
/// }
///
/// let _: Parameter<MyDsp> = Parameter::with_unit("intensity", "%", float_param!(intensity, range: 0.0..1.0, default: 0.5));
/// ```
///
/// If the field is not `f32`, it must be specified and castable to `f32`, such as:
/// `float_param!(intensity: f64, range: 0.0..1.0, default: 0.5)`
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
/// Convenience macro for quickly creating an integer parameter based on a field.
///
/// Example:
/// ```
/// use scamble::dsp::{Parameter, ParameterType};
/// use scamble::int_param;
///
/// struct MyDsp {
///     max_notes: i32
/// }
///
/// let _: Parameter<MyDsp> = Parameter::new("max_notes", int_param!(max_notes, range: 0..10, default: 1));
/// ```
///
/// If the field is not `i32`, it must be specified and castable to `i32`, such as:
/// `int_param!(max_notes: usize, range: 0..4096, default: 256)`
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
/// Convenience macro for quickly creating a boolean parameter based on a field.
///
/// Example:
/// ```
/// use scamble::dsp::{Parameter, ParameterType};
/// use scamble::bool_param;
///
/// struct MyDsp {
///     legato: bool
/// }
///
/// let _: Parameter<MyDsp> = Parameter::new("legato", bool_param!(legato, default: false));
/// ```
///
/// The field must be of type `bool`.
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
/// Convenience macro for quickly creating an enumerated integer parameter based on an enum field.
///
/// Note that this requires the `macro_metavar_expr` feature flag, though scamble relies on this
/// elsewhere.
///
/// Example:
/// ```
/// #![feature(macro_metavar_expr)]
///
/// use scamble::dsp::{Parameter, ParameterType};
/// use scamble::enum_param;
///
/// enum TransitionMode {
///     Gentle,
///     Intense,
///     Snap
/// }
///
/// struct MyDsp {
///     mode: TransitionMode
/// }
///
/// let _: Parameter<MyDsp> = Parameter::new("trans_mode", enum_param!(mode: TransitionMode, options: [Gentle, Intense, Snap], default: Snap));
/// ```
///
/// The field can be of any type with named constants, but must be specified. The options list is the
/// canonical (saved) order of those constants, so modifications are not backwards compatible, though
/// additions are. Changing the default is backwards compatible.
///
/// Default names are given by the names of constants, but plugin scripts can change the name displayed
/// in the FMOD Studio UI.
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