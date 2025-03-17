use std::thread::sleep;
use std::time::Duration;
use fmod::system::System;
use crate::fmod;
use crate::fmod::channel_group::ChannelGroup;
use crate::fmod::result::FmResultTrait;
use crate::raw_bindings::FMOD_DSP_TYPE;

#[test]
fn simulate() {
    let system = System::create().fm_unwrap();
    let desc = crate::DESC;
    //let cgroup = system.create_channel_group("group").fm_unwrap();
    // let dsp = system.create_dsp_by_type(FMOD_DSP_TYPE::FMOD_DSP_TYPE_OSCILLATOR).fm_unwrap();
    let dsp = system.create_dsp_from_description(&desc).fm_unwrap();
    let channel = system.play_dsp(&dsp, &ChannelGroup::NULL_GROUP, false).fm_unwrap();
    for _ in 0..(7*60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12));
    }
    system.release().unwrap()
}
