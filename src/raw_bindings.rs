#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::fmod::result::{FmodError, FmodResult};
use crate::raw_bindings::FMOD_RESULT::FMOD_OK;

include!(concat!(env!("OUT_DIR"), "/raw_bindings.rs"));

impl FMOD_RESULT {
    pub fn ok_then<T>(self, value: impl FnOnce() -> T) -> FmodResult<T> {
        if self == FMOD_OK {
            return Ok(value());
        }
        Err(FmodError(self))
    }
}
