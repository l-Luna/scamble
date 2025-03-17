use crate::fmod::channel::Channel;
use crate::fmod::channel_group::ChannelGroup;
use crate::fmod::dsp::DspInstance;
use crate::result::*;
use crate::raw_bindings::*;
use std::ffi::CString;
use std::ptr;

pub struct System(*mut FMOD_SYSTEM);

impl System {
    pub fn create() -> FmodResult<System> {
        let mut sys = ptr::null_mut();
        let result = unsafe { FMOD_System_Create(&mut sys, FMOD_VERSION) };
        let sys = result.ok_then(|| System(sys))?;
        unsafe { FMOD_System_Init(sys.0, 512, 0, ptr::null_mut()) }.ok_then(|| sys)
    }

    pub fn release(self) -> FmodResult<()> {
        unsafe { FMOD_System_Release(self.0) }.ok_then(|| ())
    }

    pub fn update(&self) -> FmodResult<()> {
        unsafe { FMOD_System_Update(self.0) }.ok_then(|| ())
    }

    pub fn create_channel_group(&self, name: &str) -> FmodResult<ChannelGroup> {
        let mut cgroup = ptr::null_mut();
        let result = unsafe {
            FMOD_System_CreateChannelGroup(
                self.0,
                CString::new(name).unwrap().as_ptr(),
                &mut cgroup,
            )
        };
        result.ok_then(|| ChannelGroup(cgroup))
    }

    pub fn create_dsp_by_type(&self, dsp_type: FMOD_DSP_TYPE) -> FmodResult<DspInstance>{
        let mut dsp = ptr::null_mut();
        unsafe { FMOD_System_CreateDSPByType(self.0, dsp_type, &mut dsp) }.ok_then(|| DspInstance(dsp))
    }
    
    pub fn create_dsp_from_description(&self, dsp_desc: &FMOD_DSP_DESCRIPTION) -> FmodResult<DspInstance> {
        let mut dsp = ptr::null_mut();
        unsafe { FMOD_System_CreateDSP(self.0, dsp_desc, &mut dsp) }.ok_then(|| DspInstance(dsp))
    }

    pub fn play_dsp(
        &self,
        dsp: &DspInstance,
        channel_group: &ChannelGroup,
        paused: bool,
    ) -> FmodResult<Channel> {
        let mut channel = ptr::null_mut();
        unsafe {
            FMOD_System_PlayDSP(
                self.0,
                dsp.0,
                channel_group.0,
                FMOD_BOOL::from(paused),
                &mut channel,
            )
        }
        .ok_then(|| Channel(channel))
    }
}
