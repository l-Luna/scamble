use crate::custom_dsp::{with_sidechain, Dsp, DspType, Parameter, ParameterType};

pub struct ExactOut {
    sidechain_enabled: bool
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
        vec![
            Parameter {
                ty: ParameterType::Sidechain {
                    setter: |value, dsp| dsp.sidechain_enabled = value,
                    getter: |dsp| dsp.sidechain_enabled,
                },
                name: "sidechain",
                unit: "",
                desc: "Where to receive input from",
            }
        ]
    }

    fn create() -> Self {
        ExactOut {
            sidechain_enabled: false
        }
    }

    fn reset(&mut self) {

    }

    fn read(&mut self, in_data: &[f32], out_data: &mut [f32], in_channels: usize, out_channels: usize) {
        with_sidechain(|it| match it {
            None => {}
            Some((sidechain_data, sidechain_channels)) => {
                out_data.copy_from_slice(sidechain_data)
            }
        });
    }
}