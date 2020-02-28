[![Build Status][build_img]][build_lnk] [![Docs badge]][docs.rs] [![Crates badge]][crates.io]

[build_img]: https://github.com/jaynus/spine/workflows/CI/badge.svg
[build_lnk]: https://github.com/jaynus/spine/actions
[Crates badge]: https://img.shields.io/crates/v/spine.svg
[crates.io]: https://crates.io/spine
[Docs badge]: https://img.shields.io/badge/docs.rs-rustdoc-green
[docs.rs]: https://docs.rs/spine

![Spine rendering in glow](https://github.com/jaynus/spine/raw/master/spine-example/examples/spine_glow.gif)

# Rust Spine Runtime

These are high level, safe Rust bindings for the [spine-c](https://github.com/EsotericSoftware/spine-runtimes/) runtime 
by [Esoteric Software](http://esotericsoftware.com/)


## WIP Status
* spine-c FFI Wrapper
    - [x] Proof of concept example runs and animates
    - Struct Wrappers completion
        - [ ] Skeleton
            - [ ] SkeletonData
            - [ ] Skeleton
            - [ ] SlotData
            - [ ] Slot
            - [x] Attachment
            - [x] RegionAttachment
            - [x] Bone
        - [ ] Animation
            - [ ] AnimationData
            - [ ] TaskEntry
        
        ....
        
## Code Example 

```rust
// Load the spine texture atlas
let atlas = Atlas::from_file("example.atlas", |atlas_page, path| {
    // Perform Texture loading into your renderer here. 
    // Return a u32 that will be used internally to reference the texture
    123
}).unwrap();

// Load the spine skeleton data from a binary file
let skeleton_data = SkeletonData::from_binary_file("example.skel", atlas).unwrap();
// Load the animation data from the associated skeleton
let animation_data = AnimationStateData::new(&skeleton_data);

// Spawn an instance of the skeleton animation. Each Skeleton+Animation combo references a unique skeleton and animation set.
let mut skeleton = Skeleton::new(&skeleton_data);
let mut animation = AnimationState::new(&animation_data);

// List available animations
skeleton_data.animations().iter().for_each(|a| {
    println!("Available Animation: {}", a.name());
});

// Set an active animation
animation.set_by_name(animations[0].name(), TrackIndex::zero(), true);

....

// Animations and skeletons are then updated as follows, which follows the spine runtime.
skeleton.update(delta_time_f32_seconds);

animation.update(delta_time_f32_seconds);
animation.apply(skeleton);

skeleton.update_world_transforms();

```