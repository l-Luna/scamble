use vizia::binding::{Lens, LensValue};
use scamble::fmod::dsp::DspInstance;

#[derive(Copy, Clone, Debug, Hash)]
pub struct FloatParamLens {
    idx: usize
}

impl FloatParamLens {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

impl Lens for FloatParamLens {
    type Source = DspInstance;
    type Target = f32;

    fn view<'a>(&self, source: &'a DspInstance) -> Option<LensValue<'a, f32>> {
        source.get_float_parameter(self.idx).ok().map(LensValue::Owned)
    }
}

#[derive(Copy, Clone, Debug, Hash)]
pub struct BoolParamLens {
    idx: usize
}

impl BoolParamLens {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

impl Lens for BoolParamLens {
    type Source = DspInstance;
    type Target = bool;

    fn view<'a>(&self, source: &'a DspInstance) -> Option<LensValue<'a, bool>> {
        source.get_bool_parameter(self.idx).ok().map(LensValue::Owned)
    }
}

#[derive(Copy, Clone, Debug, Hash)]
pub struct IntParamLens {
    idx: usize
}

impl IntParamLens {
    pub fn new(idx: usize) -> Self {
        Self { idx }
    }
}

impl Lens for IntParamLens {
    type Source = DspInstance;
    type Target = i32;

    fn view<'a>(&self, source: &'a DspInstance) -> Option<LensValue<'a, i32>> {
        source.get_int_parameter(self.idx).ok().map(LensValue::Owned)
    }
}