use crate::raw_bindings::FMOD_SOUND;

pub struct Sound(pub(crate) *mut FMOD_SOUND);
