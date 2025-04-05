use crate::dsp::Parameter;
use crate::dsp::interop::{Dsp, DspType, ParameterType, with_sidechain};
use crate::dsp::signal::{Signal, SignalConst, SignalMut};

pub struct ExactOut {
    sidechain_enabled: bool,
}

impl Dsp for ExactOut {
    fn name() -> &'static str {
        "Exact"
    }

    fn version() -> u32 {
        1
    }

    fn ty() -> DspType {
        DspType::Effect
    }

    fn parameters() -> Vec<Parameter<Self>> {
        vec![Parameter {
            ty: ParameterType::Sidechain {
                setter: |value, dsp| dsp.sidechain_enabled = value,
                getter: |dsp| dsp.sidechain_enabled,
            },
            name: "sidechain",
            unit: "",
            desc: "Where to receive input from",
        }]
    }

    fn create() -> Self {
        ExactOut {
            sidechain_enabled: false,
        }
    }

    fn read(&mut self, _: SignalConst, mut output: SignalMut) {
        with_sidechain(|it| match it {
            None => {}
            Some(sidechain) => output.samples_mut().copy_from_slice(sidechain.samples()),
        });
    }
}
