use crate::ffi::*;

macro_rules! spine_enum {
    ($primitive:tt, $rust_name:ident, $ffi_name:ident, $($name:ident = $value:literal, )+) => {
        #[repr($primitive)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $rust_name {
            $(
                $name = $value,
            )+
        }
        impl From<$ffi_name> for $rust_name {
            fn from(e: $ffi_name) -> Self {
                match e {
                   $(
                        $value => Self::$name,
                    )+
                    _ => panic!( "ERROR: Unsupported enum value provided for type '{}({})' value = '{}'", stringify!($rust_name), stringify!($ffi_name), e),
                }
            }
        }
    };
}

spine_enum! { u32, AtlasFormat, spAtlasFormat,
    Unknown = 0,
    Alpha = 1,
    Intensity = 2,
    LuminanceAlpha = 3,
    RGB565 = 4,
    RGBA4444 = 5,
    RGB888 = 6,
    RGBA8888 = 7,
}

spine_enum! { u32, AtlasFilter, spAtlasFilter,
    Unknown = 0,
    Nearest = 1,
    Linear = 2,
    Mipmap = 3,
    MipmapNereastNearest = 4,
    MipmapLinearNearest = 5,
    MipmapNearestLinear = 6,
    MipmapLinearLinear = 7,
}

spine_enum! { u32, AtlasWrap, spAtlasWrap,
    MirroedRepeat = 0,
    ClampToEdge = 1,
    Repeat = 2,
}

spine_enum! { u32, BlendMode, spBlendMode,
    Normal = 0,
    Additive = 1,
    Multiply = 2,
    Screen = 3,
}

spine_enum! { u32, AttachmentType, spAttachmentType,
    Region = 0,
    BoundingBox = 1,
    Mesh = 2,
    LinkedMesh = 3,
    Path = 4,
    Point = 5,
    Clipping = 6,
}

#[repr(u32)]
pub enum EventType {
    Start = 0,
    Interrupt = 1,
    End = 2,
    Complete = 3,
    Dispose = 4,
    Event = 5,
}
//pub const spEventType_SP_ANIMATION_START: spEventType = 0;
// pub const spEventType_SP_ANIMATION_INTERRUPT: spEventType = 1;
// pub const spEventType_SP_ANIMATION_END: spEventType = 2;
// pub const spEventType_SP_ANIMATION_COMPLETE: spEventType = 3;
// pub const spEventType_SP_ANIMATION_DISPOSE: spEventType = 4;
// pub const spEventType_SP_ANIMATION_EVENT: spEventType = 5;
