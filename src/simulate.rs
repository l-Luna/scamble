use std::thread::sleep;
use std::time::Duration;
use fmod::system::System;
use crate::fmod;
use crate::fmod::result::FmResultTrait;

#[test]
fn simulate() {
    let system = System::create().fm_unwrap();
    //let cgroup = system.create_channel_group("group").fm_unwrap();
    let dsp = system.create_dsp_from_description(crate::DESC).fm_unwrap();
    let channel = system.play_dsp(&dsp, &fmod::channel_group::ChannelGroup::NULL_GROUP, true).fm_unwrap();
    for _ in 0..(7*60) {
        system.update().fm_unwrap();
        sleep(Duration::from_millis(12))
    }
    system.release().unwrap()
}
