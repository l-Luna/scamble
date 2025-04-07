mod dsp_lenses;

use scamble::dsp::{Dsp, Parameter, ParameterType, interop};
use scamble::fmod::dsp::DspInstance;
use scamble::fmod::system::System;
use scamble::result::FmResultTrait;

use vizia::prelude::*;

type It = scamble::effects::fantasy::Fantasy;

#[derive(Lens)]
struct AppModel {
    system: System,
    dsp: Option<DspInstance>,
}

impl Model for AppModel {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map::<AppEvent, _>(|event, _meta| match event {
            AppEvent::SetParamFloat(idx, v) => {
                if let Some(instance) = self.dsp {
                    instance.set_float_parameter(*idx, *v).unwrap()
                }
            }
            AppEvent::SetParamBool(idx, v) => {
                if let Some(instance) = self.dsp {
                    instance.set_bool_parameter(*idx, *v).unwrap()
                }
            }
            AppEvent::SetParamInt(idx, v) => {
                if let Some(instance) = self.dsp {
                    instance.set_int_parameter(*idx, *v).unwrap()
                }
            }
            AppEvent::ToggleParamBool(idx) => {
                if let Some(instance) = self.dsp {
                    instance
                        .set_bool_parameter(*idx, !instance.get_bool_parameter(*idx).unwrap())
                        .unwrap()
                }
            }
        });
    }
}

enum AppEvent {
    SetParamFloat(usize, f32),
    SetParamBool(usize, bool),
    ToggleParamBool(usize),
    SetParamInt(usize, i32),
}

fn main() -> Result<(), ApplicationError> {
    let system = System::create().fm_unwrap();
    let desc = interop::into_desc::<It>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let sound = system.create_sound("./sound.mp3").fm_unwrap();
    let channel = system.play_sound(sound, None, true).fm_unwrap();
    channel.add_dsp(0, &dsp).fm_unwrap();
    channel.set_paused(false).fm_unwrap();

    let model = AppModel {
        system,
        dsp: Some(dsp),
    };
    Application::new(|cx| {
        model.build(cx);
        view_dsp::<It>(cx);
    })
    .title("Scamble Tester")
    .inner_size((600, 250))
    .on_idle(move |cx| system.update().unwrap())
    .run()
}

fn view_dsp<D: Dsp>(cx: &mut Context) {
    HStack::new(cx, |cx| {
        for (idx, parameter) in D::parameters().iter().enumerate() {
            view_param(cx, idx, parameter);
        }
    })
    .gap(Pixels(20.))
    .alignment(Alignment::Center);
}

fn view_param<D: Dsp>(cx: &mut Context, idx: usize, parameter: &Parameter<D>) {
    let unit: &'static str = parameter.unit;
    VStack::new(cx, |cx| {
        Label::new(cx, parameter.name);

        match parameter.ty {
            ParameterType::Float {
                min, max, default, ..
            } => {
                Knob::new(
                    cx,
                    norm(default, min, max),
                    read_float_lens(idx, default).map(move |it| norm(*it, min, max)),
                    false,
                )
                .on_change(move |cx, value| cx.emit(AppEvent::SetParamFloat(idx, value)));

                Label::new(
                    cx,
                    read_float_lens(idx, default).map(move |it| format!("{it}{unit}")),
                )
                .live(Live::Assertive);
            }
            ParameterType::Bool { default, names, .. } => {
                ToggleButton::new(
                    cx,
                    AppModel::dsp.map(move |dsp| {
                        dsp.and_then(|it| it.get_bool_parameter(idx).ok())
                            .unwrap_or(default)
                    }),
                    move |cx| {
                        Label::new(
                            cx,
                            read_bool_lens(idx, default).map(move |it| {
                                if let Some((on, off)) = names {
                                    if *it { on } else { off }
                                } else {
                                    if *it { "ON" } else { "OFF" }
                                }
                            }),
                        )
                        .live(Live::Assertive)
                    },
                )
                .on_toggle(move |cx| {
                    cx.emit(AppEvent::ToggleParamBool(idx));
                });
            }
            ParameterType::Int {
                min, max, default, ..
            } => {
                Slider::new(cx, read_int_lens(idx, default).map(|it| *it as f32))
                    .range(min as f32..max as f32)
                    .on_change(move |cx, value| {
                        cx.emit(AppEvent::SetParamInt(idx, value as i32));
                    });

                Label::new(
                    cx,
                    read_int_lens(idx, default).map(move |it| format!("{it}{unit}")),
                )
                .live(Live::Assertive);
            }
            _ => {}
        };
    })
    .gap(Pixels(10.))
    .alignment(Alignment::Center);
}

fn read_float_lens(idx: usize, default: f32) -> impl Lens<Target = f32> {
    AppModel::dsp.map(move |dsp| {
        dsp.and_then(|it| it.get_float_parameter(idx).ok())
            .unwrap_or(default)
    })
}

fn read_bool_lens(idx: usize, default: bool) -> impl Lens<Target = bool> {
    AppModel::dsp.map(move |dsp| {
        dsp.and_then(|it| it.get_bool_parameter(idx).ok())
            .unwrap_or(default)
    })
}

fn read_int_lens(idx: usize, default: i32) -> impl Lens<Target = i32> {
    AppModel::dsp.map(move |dsp| {
        dsp.and_then(|it| it.get_int_parameter(idx).ok())
            .unwrap_or(default)
    })
}

fn norm(v: f32, min: f32, max: f32) -> f32 {
    (v - min) / (max - min)
}

fn unnorm(p: f32, min: f32, max: f32) -> f32 {
    p * (max - min) + min
}
