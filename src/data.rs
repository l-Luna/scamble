#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct _3DAttributes {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
    pub up: Vec3
}

#[derive(Clone, Debug, PartialEq)]
pub struct FftData {
    pub data: Vec<Vec<f32>>
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicResponseData {
    pub rms: Vec<f32>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OverallGainData {
    pub linear_gain: f32,
    pub linear_gain_additive: f32
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ListenerAttributesData {
    pub relative: _3DAttributes,
    pub absolute: _3DAttributes
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListenerAttributesListData {
    pub count: usize,
    pub relative: Vec<_3DAttributes>,
    pub weights: Vec<f32>,
    pub absolute: _3DAttributes
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AttenuationRangeData {
    pub min: f32,
    pub max: f32
}