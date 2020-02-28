#![deny(clippy::all, clippy::pedantic)]
#![allow(dead_code, clippy::module_name_repetitions, clippy::must_use_candidate)]

use atlas::AtlasPage;
use std::ffi::CStr;
use std::{os::raw::c_char, path::Path};
use thiserror::Error;

pub mod ffi;

pub mod animation;
pub mod atlas;
pub mod enums;
pub mod skeleton;

mod spine_ptr;
use spine_ptr::*;

#[derive(Debug, Error)]
pub enum SpineError {
    #[error("LOL")]
    FailLoadAtlas(String),
    #[error("{}", 0)]
    FailLoadSkeleton(String),
}

#[allow(clippy::mut_mut)]
#[no_mangle]
extern "C" fn _spAtlasPage_createTexture(
    atlas_page_ptr: *mut ffi::spAtlasPage,
    path: *const std::os::raw::c_char,
) {
    std::panic::catch_unwind(|| {
        let path = unsafe { CStr::from_ptr(path).to_str().unwrap().to_owned() };

        let mut atlas_page = AtlasPage {
            inner: SpineMutPtr::new(atlas_page_ptr, None),
        };

        let atlas_object_ptr = unsafe { (*atlas_page.inner.as_mut().atlas).rendererObject };
        //let atlas_page_object_ptr = atlas_page.inner.as_mut().rendererObject;

        if !atlas_object_ptr.is_null() {
            let closure: &mut &mut dyn FnMut(&AtlasPage, &Path) -> u32 =
                unsafe { &mut *(atlas_object_ptr as *mut _) };

            atlas_page.set_texture_id(closure(&atlas_page, Path::new(&path)));
        }
    })
    .unwrap_or_else(|e| println!("ERROR: {:?}", e));
}

#[no_mangle]
extern "C" fn _spAtlasPage_disposeTexture(atlas: *mut ffi::spAtlasPage) {
    std::panic::catch_unwind(|| {
        let _atlas_page = AtlasPage {
            inner: SpineMutPtr::new(atlas, None),
        };

        // TODO: no-op?
    })
    .unwrap_or_else(|e| println!("ERROR: {:?}", e));
}

#[no_mangle]
unsafe extern "C" fn _spUtil_readFile(path: *const c_char, length: *mut i32) -> *mut c_char {
    std::panic::catch_unwind(|| ffi::_spReadFile(path, length)).unwrap_or_else(|e| {
        println!("ERROR: {:?}", e);
        std::ptr::null_mut()
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use std::path::PathBuf;

    pub struct TestCase {
        name: &'static str,
        atlas: &'static str,
        binary: &'static str,
        json: &'static str,
        path: &'static str,
    }
    impl TestCase {
        pub fn name(&self) -> &str {
            self.name
        }

        pub fn atlas(&self) -> PathBuf {
            PathBuf::from(self.path).join(self.atlas)
        }

        pub fn binary(&self) -> PathBuf {
            PathBuf::from(self.path).join(self.binary)
        }

        pub fn json(&self) -> PathBuf {
            PathBuf::from(self.path).join(self.json)
        }
    }

    pub const TEST_CASES: &[TestCase] = &[TestCase {
        name: "dragon",
        atlas: "dragon.atlas",
        binary: "dragon-ess.skel",
        json: "dragon-ess.json",
        path: "external/examples/dragon/export",
    }];
}
