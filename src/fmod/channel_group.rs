use crate::raw_bindings::FMOD_CHANNELGROUP;
use std::ptr;

pub struct ChannelGroup(pub(in crate::fmod) *mut FMOD_CHANNELGROUP);

impl ChannelGroup {
    pub(crate) const NULL_GROUP: ChannelGroup = ChannelGroup(ptr::null_mut());
}
