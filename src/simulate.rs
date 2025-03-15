use crate::raw_bindings::{
    FMOD_CHANNELGROUP, FMOD_SYSTEM, FMOD_System_Create, FMOD_System_CreateChannelGroup,
    FMOD_System_PlayDSP, FMOD_VERSION,
};
use std::ffi::CString;
use std::ptr;
use std::thread::sleep;
use std::time::Duration;
use fmod::system::System;
use crate::fmod;
use crate::fmod::result::FmResultTrait;

#[test]
fn simulate() {
    let system = System::create().fm_unwrap();
    let cgroup = system.create_channel_group("group").fm_unwrap();
    let dsp = system.create_dsp_from_description(crate::DESC).fm_unwrap();
    let channel = system.play_dsp(&dsp, &cgroup, false).fm_unwrap();
    for i in 0..7 {
        sleep(Duration::from_secs(1))
    }
    system.release().unwrap()
}
