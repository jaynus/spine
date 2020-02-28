#![deny(clippy::all, clippy::pedantic)]
#![allow(dead_code, clippy::module_name_repetitions, clippy::must_use_candidate)]

use std::{ffi::*, os::raw::*, path::Path, ptr};

pub mod enums;
pub mod ffi;

pub mod animation;
pub mod skeleton;

mod spine_ptr;
use spine_ptr::*;

#[derive(Debug, failure::Fail)]
pub enum SpineError {
    #[fail(display = "LOL")]
    FailLoadAtlas,
    #[fail(display = "{}", 0)]
    FailLoadSkeleton(String),
}

pub struct Atlas {
    pub(crate) inner: SpineMutPtr<ffi::spAtlas>,
}

impl Atlas {
    #[allow(clippy::mut_mut)]
    pub fn from_file<P, F>(path: P, mut create_texture: F) -> Result<Self, failure::Error>
    where
        P: AsRef<Path>,
        F: FnMut(&AtlasPage, &Path) -> u32,
    {
        let path_str_c = CString::new(
            path.as_ref()
                .to_str()
                .ok_or_else(|| failure::err_msg("Failed to convert path to string"))?,
        )?;

        // TODO:
        let mut closure_ref: &mut dyn FnMut(&AtlasPage, &Path) -> u32 = &mut create_texture;
        let trait_obj_ref: &mut &mut dyn FnMut(&AtlasPage, &Path) -> u32 = &mut closure_ref;

        let closure_pointer_pointer = trait_obj_ref as *mut _ as *mut c_void;

        let inner =
            unsafe { ffi::spAtlas_createFromFile(path_str_c.as_ptr(), closure_pointer_pointer) };
        if inner.is_null() {
            Err(SpineError::FailLoadAtlas.into())
        } else {
            Ok(Self {
                inner: SpineMutPtr::new(inner, Some(ffi::spAtlas_dispose)),
            })
        }
    }
}

pub struct AtlasPage {
    inner: SpineMutPtr<ffi::spAtlasPage>,
}
impl AtlasPage {
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.inner.as_ref().name) }
            .to_str()
            .ok()
            .unwrap()
    }

    pub fn set_texture_id(&mut self, id: u32) {
        self.inner.as_mut().rendererObject = id as *mut c_void;
    }

    pub fn texture_id(&self) -> u32 {
        self.inner.as_ref().rendererObject as u32
    }

    pub fn format(&self) -> enums::AtlasFormat {
        self.inner.as_ref().format.into()
    }

    pub fn min_filter(&self) -> enums::AtlasFilter {
        self.inner.as_ref().minFilter.into()
    }

    pub fn mag_filter(&self) -> enums::AtlasFilter {
        self.inner.as_ref().magFilter.into()
    }

    // (U, V)
    pub fn wrap(&self) -> (enums::AtlasWrap, enums::AtlasWrap) {
        let r = self.inner.as_ref();
        (r.uWrap.into(), r.vWrap.into())
    }

    pub fn dimensions(&self) -> (i32, i32) {
        let r = self.inner.as_ref();
        (r.width, r.height)
    }
}

#[allow(clippy::mut_mut)]
#[no_mangle]
extern "C" fn _spAtlasPage_createTexture(
    atlas_page_ptr: *mut ffi::spAtlasPage,
    path: *const c_char,
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
        ptr::null_mut()
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::path::PathBuf;

    pub struct TestCase {
        name: &'static str,
        atlas: &'static str,
        binary: &'static str,
        json: &'static str,
        path: &'static str,
    }
    impl TestCase {
        pub fn atlas(&self) -> PathBuf {
            PathBuf::from(self.path).join(format!("{}", self.atlas))
        }

        pub fn binary(&self) -> PathBuf {
            PathBuf::from(self.path).join(format!("{}", self.binary))
        }

        pub fn json(&self) -> PathBuf {
            PathBuf::from(self.path).join(format!("{}", self.name))
        }
    }

    pub const TEST_CASES: &'static [TestCase] = &[TestCase {
        name: "dragon",
        atlas: "dragon.atlas",
        binary: "dragon-ess.skel",
        json: "dragon-ess.json",
        path: "external/examples/dragon/export",
    }];

    #[test]
    fn load_atlas() -> Result<(), failure::Error> {
        let test_case = &TEST_CASES[0];

        let mut load = 0;

        let _ = Atlas::from_file(test_case.atlas(), |_, _| {
            load += 1;
        })?;

        assert_eq!(2, load);

        Ok(())
    }
}
