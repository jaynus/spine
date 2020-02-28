#![allow(clippy::needless_pass_by_value)]

use crate::{
    animation::Animation, enums::AttachmentType, enums::BlendMode, ffi, Atlas, SpineError,
    SpineMutPtr,
};
use std::{convert::TryInto, ffi::*, marker::PhantomData, path::Path};

pub struct BoneIndex(i32);
pub struct SlotIndex(i32);

pub struct RegionAttachment<'a> {
    pub(crate) inner: *mut ffi::spRegionAttachment,
    _lifetime: PhantomData<&'a ()>,
}
impl<'a> RegionAttachment<'a> {
    pub fn load_vertices(&self, bone: &Bone<'_>, vertices: &mut [f32; 8]) {
        unsafe {
            ffi::spRegionAttachment_computeWorldVertices(
                self.inner,
                bone.inner,
                vertices.as_mut_ptr(),
                0,
                2,
            );
        }
    }

    pub fn get_vertices(&self, bone: &Bone<'_>) -> [f32; 8] {
        let mut vertices = [0.0; 8];

        self.load_vertices(bone, &mut vertices);

        vertices
    }

    pub fn texture_id(&self) -> u32 {
        // TODO: CLEAN THIS UP OMG
        unsafe {
            let atlas_region = self.as_ref().rendererObject as *mut ffi::spAtlasRegion;
            let page = (*atlas_region).page;
            let texture_id = (*page).rendererObject as u32;
            texture_id
        }
    }

    fn rotation(&self) -> f32 {
        self.as_ref().rotation
    }

    pub fn color(&self) -> [f32; 4] {
        let color = &self.as_ref().color;
        [color.r, color.b, color.g, color.a]
    }

    pub fn position(&self) -> [f32; 2] {
        let r = self.as_ref();
        [r.x, r.y]
    }

    pub fn scale(&self) -> [f32; 2] {
        let r = self.as_ref();
        [r.scaleX, r.scaleY]
    }

    pub fn dimensions(&self) -> [f32; 2] {
        let r = self.as_ref();
        [r.width, r.height]
    }

    pub fn uv(&self) -> [f32; 8] {
        let r = self.as_ref();
        r.uvs
    }

    pub fn offset(&self) -> [f32; 8] {
        let r = self.as_ref();
        r.offset
    }

    pub(crate) fn as_ref(&self) -> &ffi::spRegionAttachment {
        unsafe { self.inner.as_ref().unwrap() }
    }
    pub(crate) fn as_mut(&mut self) -> &mut ffi::spRegionAttachment {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

pub struct Attachment<'a> {
    pub(crate) inner: *mut ffi::spAttachment,
    _lifetime: PhantomData<&'a ()>,
}
impl<'a> Attachment<'a> {
    pub fn as_region_attachment(&mut self) -> RegionAttachment<'a> {
        RegionAttachment {
            inner: self.inner as *mut ffi::spRegionAttachment,
            _lifetime: PhantomData::<&'a ()>::default(),
        }
    }

    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.as_ref().name) }
            .to_str()
            .unwrap()
    }

    pub fn kind(&self) -> AttachmentType {
        self.as_ref().type_.into()
    }

    pub(crate) fn as_ref(&self) -> &ffi::spAttachment {
        unsafe { self.inner.as_ref().unwrap() }
    }
    pub(crate) fn as_mut(&mut self) -> &mut ffi::spAttachment {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

pub struct Slot<'a> {
    pub(crate) inner: *mut ffi::spSlot,
    _lifetime: PhantomData<&'a ()>,
}
impl<'a> Slot<'a> {
    pub fn bone(&self) -> Option<Bone<'_>> {
        let r = self.as_ref();

        if r.bone.is_null() {
            None
        } else {
            Some(Bone {
                inner: r.bone,
                _lifetime: PhantomData::<&'a ()>::default(),
            })
        }
    }

    pub fn color(&self) -> [f32; 4] {
        let color = &self.as_ref().color;
        [color.r, color.b, color.g, color.a]
    }

    pub fn blend_mode(&self) -> BlendMode {
        unsafe { self.as_ref().data.as_ref() }
            .unwrap()
            .blendMode
            .into()
    }

    pub fn active_attachment(&self) -> Option<Attachment<'_>> {
        let r = self.as_ref();

        if r.attachment.is_null() {
            None
        } else {
            Some(Attachment {
                inner: r.attachment,
                _lifetime: PhantomData::<&'a ()>::default(),
            })
        }
    }

    pub fn index(&self) -> SlotIndex {
        unsafe { SlotIndex((*self.as_ref().data).index) }
    }

    pub(crate) fn as_ref(&self) -> &ffi::spSlot {
        unsafe { self.inner.as_ref().unwrap() }
    }
    pub(crate) fn as_mut(&mut self) -> &mut ffi::spSlot {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

pub struct Bone<'a> {
    pub(crate) inner: *mut ffi::spBone,
    _lifetime: PhantomData<&'a ()>,
}
impl<'a> Bone<'a> {
    pub fn index(&self) -> BoneIndex {
        unsafe { BoneIndex((*self.as_ref().data).index) }
    }

    pub(crate) fn as_ref(&self) -> &ffi::spBone {
        unsafe { self.inner.as_ref().unwrap() }
    }
    pub(crate) fn as_mut(&mut self) -> &mut ffi::spBone {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

pub struct BoneData<'a> {
    pub(crate) inner: &'a ffi::spBoneData,
}
impl<'a> BoneData<'a> {
    pub fn name(&self) -> &str {
        unsafe { CStr::from_ptr(self.inner.name) }.to_str().unwrap()
    }
}
impl<'a> std::fmt::Debug for BoneData<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Bone {{ name: {} }}", self.name())
    }
}

pub struct Skeleton {
    pub(crate) inner: SpineMutPtr<ffi::spSkeleton>,
}

impl Skeleton {
    pub fn new(data: &SkeletonData) -> Self {
        Self {
            inner: SpineMutPtr::new(
                unsafe { ffi::spSkeleton_create(data.inner.as_mut_ptr()) },
                Some(ffi::spSkeleton_dispose),
            ),
        }
    }

    pub fn slots(&self) -> Vec<Slot<'_>> {
        let inner_ref = self.inner.as_ref();
        let mut converted = Vec::with_capacity(inner_ref.slotsCount.try_into().unwrap());

        for n in 0..inner_ref.slotsCount.try_into().unwrap() {
            unsafe {
                let inner = *(inner_ref.slots.add(n));
                converted.push(Slot {
                    inner,
                    _lifetime: PhantomData::<&'_ ()>::default(),
                });
            }
        }

        converted
    }

    pub fn draw_slots(&self) -> Vec<Slot<'_>> {
        let inner_ref = self.inner.as_ref();
        let mut converted = Vec::with_capacity(inner_ref.slotsCount.try_into().unwrap());

        for n in 0..inner_ref.slotsCount.try_into().unwrap() {
            unsafe {
                let inner = *(inner_ref.drawOrder.add(n));
                converted.push(Slot {
                    inner,
                    _lifetime: PhantomData::<&'_ ()>::default(),
                });
            }
        }

        converted
    }

    pub fn bone(&self) -> Vec<Bone<'_>> {
        let inner_ref = self.inner.as_ref();
        let mut converted = Vec::with_capacity(inner_ref.bonesCount.try_into().unwrap());

        for n in 0..inner_ref.bonesCount.try_into().unwrap() {
            unsafe {
                let inner = *(inner_ref.bones.add(n));
                converted.push(Bone {
                    inner,
                    _lifetime: PhantomData::<&'_ ()>::default(),
                });
            }
        }

        converted
    }

    pub fn bone_index(&self, name: &str) -> BoneIndex {
        let name = CString::new(name).unwrap();
        BoneIndex(unsafe { ffi::spSkeleton_findBoneIndex(self.inner.as_mut_ptr(), name.as_ptr()) })
    }

    pub fn slot(&self, name: &str) -> Option<Slot<'_>> {
        let name = CString::new(name).unwrap();
        let inner = unsafe { ffi::spSkeleton_findSlot(self.inner.as_mut_ptr(), name.as_ptr()) };

        if inner.is_null() {
            None
        } else {
            Some(Slot {
                inner,
                _lifetime: PhantomData::<&'_ ()>::default(),
            })
        }
    }

    pub fn color(&self) -> [f32; 4] {
        let color = &self.inner.as_ref().color;
        [color.r, color.b, color.g, color.a]
    }

    pub fn position(&self) -> [f32; 2] {
        let r = self.inner.as_ref();
        [r.x, r.y]
    }

    pub fn scale(&self) -> [f32; 2] {
        let r = self.inner.as_ref();
        [r.scaleX, r.scaleY]
    }

    fn time(&self) -> f32 {
        self.inner.as_ref().time
    }

    pub fn slot_index(&self, name: &str) -> SlotIndex {
        let name = CString::new(name).unwrap();
        SlotIndex(unsafe { ffi::spSkeleton_findSlotIndex(self.inner.as_mut_ptr(), name.as_ptr()) })
    }

    pub fn reset(&mut self) {
        unsafe { ffi::spSkeleton_setToSetupPose(self.inner.as_mut_ptr()) }
    }

    pub fn reset_bones(&mut self) {
        unsafe { ffi::spSkeleton_setBonesToSetupPose(self.inner.as_mut_ptr()) }
    }

    pub fn reset_slots(&mut self) {
        unsafe { ffi::spSkeleton_setSlotsToSetupPose(self.inner.as_mut_ptr()) }
    }

    pub fn update_cache(&mut self) {
        unsafe { ffi::spSkeleton_updateCache(self.inner.as_mut_ptr()) }
    }

    pub fn update(&mut self, delta: f32) {
        unsafe { ffi::spSkeleton_update(self.inner.as_mut_ptr(), delta) }
    }

    pub fn update_world_transform(&mut self) {
        unsafe { ffi::spSkeleton_updateWorldTransform(self.inner.as_mut_ptr()) }
    }
}

pub struct SkeletonData {
    pub(crate) inner: SpineMutPtr<ffi::spSkeletonData>,
    pub(crate) atlas: SpineMutPtr<ffi::spAtlas>,
}

impl SkeletonData {
    pub(crate) fn as_mut(&mut self) -> &mut ffi::spSkeletonData {
        self.inner.as_mut()
    }
    pub(crate) fn as_ref(&self) -> &ffi::spSkeletonData {
        self.inner.as_ref()
    }

    pub fn animations<'a>(&'a self) -> Vec<Animation<'a>> {
        let mut converted = Vec::with_capacity(self.as_ref().animationsCount.try_into().unwrap());

        for n in 0..self.as_ref().animationsCount.try_into().unwrap() {
            unsafe {
                let inner = *(self.as_ref().animations.add(n));
                converted.push(Animation {
                    inner,
                    _lifetime: PhantomData::<&'a ()>::default(),
                });
            }
        }

        converted
    }

    pub fn bones<'a>(&'a self) -> Vec<BoneData<'a>> {
        let mut converted = Vec::with_capacity(self.as_ref().bonesCount.try_into().unwrap());

        for n in 0..self.as_ref().bonesCount.try_into().unwrap() {
            unsafe {
                let inner = *(self.as_ref().bones.add(n));
                converted.push(BoneData {
                    inner: inner.as_mut().unwrap(),
                });
            }
        }

        converted
    }

    pub fn strings(&self) -> Vec<&str> {
        let mut converted = Vec::new();

        for n in 0..self.as_ref().stringsCount.try_into().unwrap() {
            unsafe {
                converted.push(
                    CStr::from_ptr(*self.as_ref().strings.add(n))
                        .to_str()
                        .unwrap(),
                );
            }
        }

        converted
    }

    pub fn position(&self) -> (f32, f32) {
        let r = self.inner.as_ref();
        (r.x, r.y)
    }
    pub fn dimensions(&self) -> (f32, f32) {
        let r = self.inner.as_ref();
        (r.width, r.height)
    }
    pub fn set_position(&mut self, position: (f32, f32)) {
        let r = self.inner.as_mut();
        r.x = position.0;
        r.y = position.1;
    }
    pub fn set_dimensions(&mut self, dimensions: (f32, f32)) {
        let r = self.inner.as_mut();
        r.width = dimensions.0;
        r.height = dimensions.1;
    }

    pub fn from_binary_file<P>(path: P, atlas: Atlas) -> Result<Self, failure::Error>
    where
        P: AsRef<Path>,
    {
        let path_str_c = CString::new(
            path.as_ref()
                .to_str()
                .ok_or_else(|| failure::err_msg("Failed to convert path to string"))?,
        )?;

        unsafe {
            let binary_data = ffi::spSkeletonBinary_create(atlas.inner.as_mut_ptr());
            if binary_data.is_null() {
                return Err(SpineError::FailLoadSkeleton(
                    "failed to begin binary data load".to_owned(),
                )
                .into());
            }
            let data = ffi::spSkeletonBinary_readSkeletonDataFile(binary_data, path_str_c.as_ptr());
            if data.is_null() {
                ffi::spSkeletonBinary_dispose(binary_data);
                return Err(SpineError::FailLoadSkeleton(
                    CStr::from_ptr((*binary_data).error).to_str()?.to_owned(),
                )
                .into());
            }

            ffi::spSkeletonBinary_dispose(binary_data);

            Ok(Self {
                inner: SpineMutPtr::new(data, Some(ffi::spSkeletonData_dispose)),
                atlas: atlas.inner,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TEST_CASES;

    #[test]
    fn load_skeleton() -> Result<(), failure::Error> {
        let test_case = &TEST_CASES[0];

        let mut load = 0;

        let atlas = Atlas::from_file(test_case.atlas(), |_, _| {
            load += 1;
        })?;

        assert_eq!(2, load);

        let skeleton_data = SkeletonData::from_binary_file(test_case.binary(), atlas)?;
        let _skeleton = Skeleton::new(&skeleton_data);
        println!("bones = {:?}", skeleton_data.bones());

        Ok(())
    }

    #[test]
    fn load_skeleton_drop_order() -> Result<(), failure::Error> {
        let test_case = &TEST_CASES[0];

        let mut load = 0;

        let skeleton_data = {
            let atlas = Atlas::from_file(test_case.atlas(), |_, _| {
                load += 1;
            })?;

            assert_eq!(2, load);

            SkeletonData::from_binary_file(test_case.binary(), atlas)?
        };
        let _skeleton = Skeleton::new(&skeleton_data);
        println!("bones = {:?}", skeleton_data.bones());

        Ok(())
    }
}
