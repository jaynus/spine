#![allow(clippy::needless_pass_by_value)]

use crate::{
    ffi,
    skeleton::{Skeleton, SkeletonData},
    SpineMutPtr,
};
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

pub struct TrackIndex(i32);
impl TrackIndex {
    pub fn zero() -> Self {
        Self(0)
    }
}
impl Default for TrackIndex {
    fn default() -> Self {
        Self::zero()
    }
}

pub struct Animation<'a> {
    pub(crate) inner: *mut ffi::spAnimation,
    pub(crate) _lifetime: PhantomData<&'a ()>,
}
impl<'a> Animation<'a> {
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.inner.as_ref().unwrap().name) }
            .to_str()
            .unwrap()
    }

    pub fn duration(&self) -> f32 {
        unsafe { self.inner.as_ref().unwrap().duration }
    }
    //    pub fn apply(&self, skeleton: &mut Skeleton, last_time: f32, time: f32, loop_: i32) {}
}

pub struct AnimationState {
    pub(crate) inner: SpineMutPtr<ffi::spAnimationState>,
    pub(crate) parent: SpineMutPtr<ffi::spAnimationStateData>,
}
impl AnimationState {
    pub fn new(data: &AnimationStateData) -> Self {
        Self {
            inner: SpineMutPtr::new(
                unsafe { ffi::spAnimationState_create(data.inner.as_mut_ptr()) },
                Some(ffi::spAnimationState_dispose),
            ),
            parent: data.inner.clone(),
        }
    }

    pub fn update(&mut self, delta: f32) {
        unsafe {
            ffi::spAnimationState_update(self.inner.as_mut_ptr(), delta);
        }
    }

    pub fn clear_track(&self, track: TrackIndex) {
        unsafe {
            ffi::spAnimationState_clearTrack(self.inner.as_mut_ptr(), track.0);
        }
    }

    pub fn clear(&self) {
        unsafe {
            ffi::spAnimationState_clearTracks(self.inner.as_mut_ptr());
        }
    }

    pub fn set(&mut self, animation: &Animation, track_index: TrackIndex, do_loop: bool) {
        unsafe {
            let _track_entry = ffi::spAnimationState_setAnimation(
                self.inner.as_mut_ptr(),
                track_index.0,
                animation.inner,
                do_loop as std::os::raw::c_int,
            );
        }
    }

    pub fn set_by_name(&mut self, animation_name: &str, track_index: TrackIndex, do_loop: bool) {
        let name = CString::new(animation_name).unwrap();
        unsafe {
            let track_entry = ffi::spAnimationState_setAnimationByName(
                self.inner.as_mut_ptr(),
                track_index.0,
                name.as_ptr(),
                do_loop as std::os::raw::c_int,
            );
            if track_entry.is_null() {
                panic!("Failed to set animation?");
            }
        }
    }

    pub fn apply(&self, skeleton: &mut Skeleton) -> bool {
        unsafe {
            ffi::spAnimationState_apply(self.inner.as_mut_ptr(), skeleton.inner.as_mut_ptr()) != 0
        }
    }
}

pub struct AnimationStateData {
    pub(crate) inner: SpineMutPtr<ffi::spAnimationStateData>,
    pub(crate) parent: SpineMutPtr<ffi::spSkeletonData>,
}
impl AnimationStateData {
    pub fn new(data: &SkeletonData) -> Self {
        Self {
            inner: SpineMutPtr::new(
                unsafe { ffi::spAnimationStateData_create(data.inner.as_mut_ptr()) },
                Some(ffi::spAnimationStateData_dispose),
            ),
            parent: data.inner.clone(),
        }
    }

    fn get_mix(&mut self, from: &Animation, to: &Animation) -> f32 {
        unsafe { ffi::spAnimationStateData_getMix(self.inner.as_mut_ptr(), from.inner, to.inner) }
    }

    fn set_mix(&mut self, from: &Animation, to: &Animation, mix: f32) {
        unsafe {
            ffi::spAnimationStateData_setMix(self.inner.as_mut_ptr(), from.inner, to.inner, mix)
        }
    }

    fn set_mix_by_name(&mut self, from: &str, to: &str, mix: f32) {
        let from = CString::new(from).unwrap();
        let to = CString::new(to).unwrap();

        unsafe {
            ffi::spAnimationStateData_setMixByName(
                self.inner.as_mut_ptr(),
                from.as_ptr(),
                to.as_ptr(),
                mix,
            )
        }
    }
}
