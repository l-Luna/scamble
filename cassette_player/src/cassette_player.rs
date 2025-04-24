use circular_buffer::CircularBuffer;
use scamble::dsp::decode::decode_into;
use scamble::dsp::signal::{Signal, SignalConst, SignalMut};
use scamble::dsp::{Dsp, DspType, Parameter, ParameterType, ProcessResult};
use scamble::{bool_param, enum_param, float_param, int_param};

#[derive(Copy, Clone)]
enum VoiceMode {
    Combine,
    Average,
    Overtake,
}

#[derive(Copy, Clone)]
struct TrailingNote {
    pos: usize,
    end: usize,
    fadeout_samples: usize,
}

pub struct CassettePlayer {
    // user-specified parameters
    samples: Vec<f32>, // downsample to mono for now
    num_notes: usize,
    _async: bool,
    start_offset_percent: f32,
    end_offset_percent: f32,
    voices: u8,
    voice_mode: VoiceMode,
    // game-state parameters
    note_frac: f32,
    // state
    trailing_notes: CircularBuffer<8, TrailingNote>,
    prev_note_frac: f32,
    async_note_tick: usize,
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

    fn approx_note_len(&self) -> usize {
        ((self.content_len() as f32) / (self.num_notes as f32)) as usize
    }

    fn note_at_pos(&self, pos: f32) -> TrailingNote {
        let clen = self.content_len() as f32;
        let start = (clen * pos) as usize + self.start_idx();
        let end = start + (clen / self.num_notes as f32) as usize;
        TrailingNote {
            pos: start,
            end,
            fadeout_samples: 0,
        }
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
                        dsp.samples.clear();
                        decode_into(data, &mut dsp.samples);
                    },
                    getter: |_| None,
                },
                name: "samples",
                unit: "",
                desc: "",
            },
            Parameter::new("num_notes", int_param!(num_notes: usize, range: 1..4096, default: 256)),
            Parameter::new("async", bool_param!(_async, default: false)),
            Parameter::with_unit("start_offset", "%", float_param!(start_offset_percent, range: 0.0..100.0, default: 0.)),
            Parameter::with_unit("end_offset", "%", float_param!(end_offset_percent, range: 0.0..100.0, default: 100.)),
            Parameter::new("voices", int_param!(voices: u8, range: 1..8, default: 4)),
            Parameter::new(
                "voice_mode",
                enum_param!(voice_mode: VoiceMode, options: [Combine, Average, Overtake], default: Combine),
            ),
            Parameter::new("note", float_param!(note_frac, range: 0.0..1.0, default: 0.)),
        ]
    }

    fn create() -> Self {
        CassettePlayer {
            samples: vec![],
            num_notes: 0,
            _async: false,
            start_offset_percent: 0.0,
            end_offset_percent: 100.0,
            voices: 4,
            voice_mode: VoiceMode::Combine,
            note_frac: 0.,
            trailing_notes: Default::default(),
            prev_note_frac: 2., // make sure the initial beat always triggers
            async_note_tick: 0,
        }
    }

    fn reset(&mut self) {
        self.trailing_notes.clear();
        self.prev_note_frac = 2.;
    }

    fn should_process(&mut self, _: bool, _: usize) -> ProcessResult {
        ProcessResult::Continue
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
            if self._async {
                // in async mode, run an internal timer disconnected from the input parameter
                // (parameter changing is all that matter)
                let async_note_frac = (self.async_note_tick as f32) / (self.num_notes as f32);
                self.trailing_notes.push_back(self.note_at_pos(async_note_frac));
                self.async_note_tick += 1;
                self.async_note_tick %= self.num_notes;
            } else {
                // in sync mode, use the position of the input as the position of the note
                self.trailing_notes.push_back(self.note_at_pos(self.note_frac));
            }
            if self.trailing_notes.len() > self.voices as usize {
                self.trailing_notes.pop_front();
            }
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
