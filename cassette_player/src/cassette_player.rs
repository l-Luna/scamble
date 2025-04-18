use scamble::dsp::signal::{Signal, SignalConst, SignalMut};
use scamble::dsp::{Dsp, DspType, Parameter, ParameterType};
use scamble::dsp::decode::decode_into;
use circular_buffer::CircularBuffer;

#[derive(Copy, Clone)]
struct TrailingNote {
    pos: usize,
    end: usize,
}

pub struct CassettePlayer {
    // user-specified parameters
    samples: Vec<f32>, // downsample to mono for now
    num_notes: usize,
    start_offset_percent: f32,
    end_offset_percent: f32,
    // game-state parameters
    note_frac: f32,
    // state
    trailing_notes: CircularBuffer<4, TrailingNote>,
    prev_note_frac: f32,
}

impl CassettePlayer {
    fn start_idx(&self) -> usize {
        let samples_f = self.samples.len() as f32;
        (samples_f * self.start_offset_percent).clamp(0., samples_f) as usize
    }

    fn end_idx(&self) -> usize {
        let samples_f = self.samples.len() as f32;
        (samples_f * self.end_offset_percent).clamp(0., samples_f) as usize
    }

    fn content_len(&self) -> usize {
        self.end_idx().saturating_sub(self.start_idx())
    }

    fn note_at_pos(&self, pos: f32) -> TrailingNote {
        let clen = self.content_len() as f32;
        let start = (clen * pos) as usize + self.start_idx();
        let end = start + (clen / self.num_notes as f32) as usize;
        TrailingNote { pos: start, end }
    }
}

impl Dsp for CassettePlayer {
    fn name() -> &'static str {
        "CassettePlayer"
    }

    fn version() -> u32 {
        1
    }

    fn ty() -> DspType {
        DspType::Generator
    }

    fn parameters() -> Vec<Parameter<Self>> {
        vec![
            Parameter {
                ty: ParameterType::Data {
                    setter: |data, dsp| {
                        // TODO: actually decode
                        dsp.samples.clear();
                        decode_into(data, &mut dsp.samples);
                    },
                    getter: |_| None,
                },
                name: "samples",
                unit: "",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Int {
                    min: 1,
                    max: 4096,
                    default: 256,
                    max_is_inf: false,
                    names: None,
                    setter: |value, dsp| dsp.num_notes = value as usize,
                    getter: |dsp| dsp.num_notes as i32,
                },
                name: "num_notes",
                unit: "",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Float {
                    min: 0.0,
                    max: 100.0,
                    default: 0.0,
                    setter: |value, dsp| dsp.start_offset_percent = value,
                    getter: |dsp| dsp.start_offset_percent,
                },
                name: "start_offset",
                unit: "%",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Float {
                    min: 0.0,
                    max: 100.0,
                    default: 100.0,
                    setter: |value, dsp| dsp.end_offset_percent = value,
                    getter: |dsp| dsp.end_offset_percent,
                },
                name: "end_offset",
                unit: "%",
                desc: "",
            },
            Parameter {
                ty: ParameterType::Float {
                    min: 0.,
                    max: 1.,
                    default: 0.,
                    setter: |value, dsp| dsp.note_frac = value,
                    getter: |dsp| dsp.note_frac,
                },
                name: "note",
                unit: "",
                desc: "",
            },
        ]
    }

    fn create() -> Self {
        CassettePlayer {
            samples: vec![],
            num_notes: 0,
            start_offset_percent: 0.0,
            end_offset_percent: 100.0,
            note_frac: 0.,
            trailing_notes: Default::default(),
            prev_note_frac: 2., // make sure the initial beat always triggers
        }
    }

    fn reset(&mut self) {
        self.trailing_notes.clear();
        self.prev_note_frac = 2.;
    }

    fn preferred_out_channels(&self) -> Option<usize> {
        Some(2)
    }

    fn read(&mut self, _: SignalConst, mut output: SignalMut) {
        // zero output, just in case
        // TODO: should scamble do ^ automatically?
        output.samples_mut().fill(0.);

        // trigger new notes when parameter changes
        if self.note_frac != self.prev_note_frac {
            self.trailing_notes.push_back(self.note_at_pos(self.note_frac));
        }
        self.prev_note_frac = self.note_frac;

        // write out existing notes to output
        if self.trailing_notes.len() > 0 {
            // hold onto output channel layout
            let out_len = output.length();
            let out_chan = output.channels();
            // reversed indexed loop to allow removal in loop
            for nidx in (0..self.trailing_notes.len()).rev() {
                let note = &mut self.trailing_notes[nidx];
                // number of samples to actually write
                let note_len = note.end.saturating_sub(note.pos).min(out_len);
                for i in 0..note_len {
                    // use `output` as a buffer; directly read samples since we know it's always
                    // mono (currently)
                    let old = output.samples_mut()[i * out_chan];
                    output.write_sample(i, old + self.samples[i + note.pos]);
                }
                // move note forward, remove empty notes
                note.pos += note_len;
                if note.end.saturating_sub(note.pos) == 0 {
                    self.trailing_notes.remove(nidx);
                }
            }
        }
    }
}
