//! Data types used as DSP parameters.

/// A 3D vector. Part of DSP parameter types.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

/// A perspective in 3D. Part of DSP parameter types.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct _3DAttributes {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
    pub up: Vec3
}

/// Multi-channel spectra, for use with [dsp::ParameterType::Fft].
#[derive(Clone, Debug, PartialEq)]
pub struct FftData {
    /// Per-channel spectra, as a vector of per-channel data.
    pub data: Vec<Vec<f32>>
}

/// Dynamic response, for use with [dsp::ParameterType::DynamicResponse].
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicResponseData {
    /// Per-channel average (RMS) gain factor.
    pub rms: Vec<f32>
}

/// Overall gain, for use with [dsp::ParameterType::OverallGain]. FMOD reads this parameter type
/// to determine when to virtualize voices.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OverallGainData {
    /// Overall linear gain applied to the signal.
    pub linear_gain: f32,
    /// Overall additive gain applied to the signal.
    pub linear_gain_additive: f32
}

/// Listener attributes, for use with [dsp::ParameterType::ListenerAttributes]. FMOD sets this
/// parameter type based on the listener's and event's locations automatically.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ListenerAttributesData {
    /// Relative position of the sound to the listener.
    pub relative: _3DAttributes,
    /// Absolute position of the sound in the world.
    pub absolute: _3DAttributes
}

/// Listener attributes when multiple listeners exist, for use with [dsp::ParameterType::ListenerAttributesList].
/// FMOD sets this parameter type based on the listeners' and event's locations automatically.
#[derive(Clone, Debug, PartialEq)]
pub struct ListenerAttributesListData {
    /// The number of listeners, equal to the length of the `relative` and `weights` fields.
    pub count: usize,
    /// Relative position of the sound to each listener.
    pub relative: Vec<_3DAttributes>,
    /// Weight of each listener to the sound output, in the range 0..1.
    pub weights: Vec<f32>,
    /// Absolute position of the sound in the world.
    pub absolute: _3DAttributes
}

/// Attenuation range of an event, for use with [dsp::ParameterType::AttenuationRange]. FMOD Studio
/// sets and updates this parameter type based on the event's minimum and maximum distance.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AttenuationRangeData {
    /// Minimum distance for attenuation.
    pub min: f32,
    /// Maximum distance for attenuation.
    pub max: f32
}