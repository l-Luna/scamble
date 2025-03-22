use crate::raw_bindings::FMOD_SOUND;

pub struct Sound(pub(in crate) *mut FMOD_SOUND);