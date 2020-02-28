use crate::{
    enums::{AtlasFilter, AtlasFormat, AtlasWrap},
    ffi,
    spine_ptr::SpineMutPtr,
    SpineError,
};
use std::{
    ffi::{CStr, CString},
    path::Path,
};

pub struct Atlas {
    pub(crate) inner: SpineMutPtr<ffi::spAtlas>,
}

impl Atlas {
    /// Loads a `Atlas` instance from the provided file path.
    ///
    /// # Errors
    /// Returns a `SpineError::FailLoadAtlas` instance, with a text message detailing why loading failed.
    #[allow(clippy::mut_mut)]
    pub fn from_file<P, F>(path: P, mut create_texture: F) -> Result<Self, SpineError>
    where
        P: AsRef<Path>,
        F: FnMut(&AtlasPage, &Path) -> u32,
    {
        let path_str_c = CString::new(path.as_ref().to_str().ok_or_else(|| {
            SpineError::FailLoadAtlas("Failed to convert path to string".to_owned())
        })?)
        .map_err(|e| {
            SpineError::FailLoadAtlas(format!("Failed to convert path to string: {:?}", e))
        })?;

        // TODO:
        let mut closure_ref: &mut dyn FnMut(&AtlasPage, &Path) -> u32 = &mut create_texture;
        let trait_obj_ref: &mut &mut dyn FnMut(&AtlasPage, &Path) -> u32 = &mut closure_ref;

        let closure_pointer_pointer = trait_obj_ref as *mut _ as *mut std::os::raw::c_void;

        let inner =
            unsafe { ffi::spAtlas_createFromFile(path_str_c.as_ptr(), closure_pointer_pointer) };
        if inner.is_null() {
            Err(SpineError::FailLoadAtlas(
                "spAtlas_createFromFile failed".to_owned(),
            ))
        } else {
            Ok(Self {
                inner: SpineMutPtr::new(inner, Some(ffi::spAtlas_dispose)),
            })
        }
    }
}

pub struct AtlasPage {
    pub(crate) inner: SpineMutPtr<ffi::spAtlasPage>,
}
impl AtlasPage {
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.inner.as_ref().name) }
            .to_str()
            .ok()
            .unwrap()
    }

    pub fn set_texture_id(&mut self, id: u32) {
        self.inner.as_mut().rendererObject = id as *mut std::os::raw::c_void;
    }

    pub fn texture_id(&self) -> u32 {
        self.inner.as_ref().rendererObject as u32
    }

    pub fn format(&self) -> AtlasFormat {
        self.inner.as_ref().format.into()
    }

    pub fn min_filter(&self) -> AtlasFilter {
        self.inner.as_ref().minFilter.into()
    }

    pub fn mag_filter(&self) -> AtlasFilter {
        self.inner.as_ref().magFilter.into()
    }

    // (U, V)
    pub fn wrap(&self) -> (AtlasWrap, AtlasWrap) {
        let r = self.inner.as_ref();
        (r.uWrap.into(), r.vWrap.into())
    }

    pub fn dimensions(&self) -> (i32, i32) {
        let r = self.inner.as_ref();
        (r.width, r.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn load_atlas() {
        let test_case = &TEST_CASES[0];

        let mut load = 0;

        let _ = Atlas::from_file(test_case.atlas(), |_, _| {
            load += 1;
            0
        })
        .unwrap();

        assert_eq!(2, load);
    }
}
