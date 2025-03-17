use std::thread::sleep;
use std::time::Duration;
use fmod::system::System;
use crate::{example, fmod};
use crate::fmod::channel_group::ChannelGroup;
use crate::custom_dsp;
use crate::result::FmResultTrait;

#[test]
fn simulate() {
    let system = System::create().fm_unwrap();
    let desc = custom_dsp::into_desc::<example::DySynth>();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let channel = system.play_dsp(&dsp, &ChannelGroup::NULL_GROUP, false).fm_unwrap();
    for _ in 0..(7*60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap()
}
