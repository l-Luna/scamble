use crate::raw_bindings::FMOD_CHANNEL;

pub struct Channel(pub(in crate::fmod) *mut FMOD_CHANNEL);
