use glow::*;
use smallvec::SmallVec;
use spine::{
    self as spine,
    animation::{AnimationState, AnimationStateData, TrackIndex},
    atlas::Atlas,
    enums::BlendMode,
    skeleton::{Skeleton, SkeletonData},
};
use std::{path::PathBuf, time::Instant};

#[repr(C)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

#[cfg(all(target_arch = "wasm32", feature = "web-sys"))]
use wasm_bindgen::prelude::*;

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
    path: "spine-sys/external/examples/dragon/export",
}];

#[cfg_attr(all(target_arch = "wasm32", feature = "web-sys"), wasm_bindgen(start))]
#[allow(clippy::main_recursion)]
pub fn wasm_main() {
    main();
}

pub trait GlowConversion {
    type Output;

    fn glow(&self) -> Self::Output;
}

impl GlowConversion for spine::enums::AtlasFilter {
    type Output = u32;

    fn glow(&self) -> Self::Output {
        match *self {
            Self::Nearest => glow::NEAREST,
            Self::Linear => glow::LINEAR,
            Self::Mipmap => glow::MIPMAP,
            Self::MipmapNereastNearest => glow::NEAREST_MIPMAP_NEAREST,
            Self::MipmapLinearNearest => glow::NEAREST_MIPMAP_NEAREST,
            Self::MipmapNearestLinear => glow::NEAREST_MIPMAP_LINEAR,
            Self::MipmapLinearLinear => glow::LINEAR_MIPMAP_LINEAR,
            _ => panic!("Unsupported"),
        }
    }
}

impl GlowConversion for spine::enums::AtlasWrap {
    type Output = u32;

    fn glow(&self) -> Self::Output {
        match *self {
            Self::MirroedRepeat => glow::MIRRORED_REPEAT,
            Self::ClampToEdge => glow::CLAMP_TO_EDGE,
            Self::Repeat => glow::REPEAT,
        }
    }
}

unsafe fn apply_blend_mode(gl: &glow::Context, blend_mode: BlendMode) {
    gl.enable(glow::BLEND);
    match blend_mode {
        BlendMode::Normal => gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA),
        BlendMode::Additive => gl.blend_func(glow::SRC_ALPHA, glow::ONE),
        BlendMode::Multiply => gl.blend_func(glow::DST_COLOR, glow::ONE_MINUS_SRC_ALPHA),
        BlendMode::Screen => gl.blend_func(glow::ONE, glow::ONE_MINUS_SRC_ALPHA),
    }
}

unsafe fn as_u8_slice<T>(slice: &[T]) -> &[u8] {
    std::slice::from_raw_parts(
        slice.as_ptr() as *const u8,
        std::mem::size_of::<T>() * slice.len(),
    )
}

unsafe fn draw_skeleton(
    skeleton: &mut Skeleton,
    animation: &mut AnimationState,
    gl: &glow::Context,
    delta: f32,
) {
    //const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

    let mut vertices = SmallVec::<[Vertex; 2048]>::default();

    skeleton.update(delta);

    animation.update(delta);
    animation.apply(skeleton);

    skeleton.update_world_transforms();

    for slot in skeleton.draw_slots() {
        apply_blend_mode(gl, slot.blend_mode());

        let skeleton_color = skeleton.color();
        let base_slot_color = slot.color();
        let slot_color = [
            skeleton_color[0] * base_slot_color[0],
            skeleton_color[1] * base_slot_color[1],
            skeleton_color[2] * base_slot_color[2],
            skeleton_color[3] * base_slot_color[3],
        ];

        if let Some(mut attachment) = slot.active_attachment() {
            match attachment.kind() {
                spine::enums::AttachmentType::Region => {
                    let region_attachment = attachment.as_region_attachment();
                    let attachment_color = region_attachment.color();

                    let color = [
                        slot_color[0] * attachment_color[0],
                        slot_color[1] * attachment_color[1],
                        slot_color[2] * attachment_color[2],
                        slot_color[3] * attachment_color[3],
                    ];

                    let texture_id = region_attachment.texture_id();

                    let positions = region_attachment.get_vertices(&slot.bone().unwrap());
                    let uvs = region_attachment.uv();

                    vertices.push(Vertex {
                        position: [positions[0], positions[1]],
                        uv: [uvs[0], uvs[1]],
                        color,
                    });
                    vertices.push(Vertex {
                        position: [positions[2], positions[3]],
                        uv: [uvs[2], uvs[3]],
                        color,
                    });
                    vertices.push(Vertex {
                        position: [positions[4], positions[5]],
                        uv: [uvs[4], uvs[5]],
                        color,
                    });
                    vertices.push(Vertex {
                        position: [positions[4], positions[5]],
                        uv: [uvs[4], uvs[5]],
                        color,
                    });
                    vertices.push(Vertex {
                        position: [positions[6], positions[7]],
                        uv: [uvs[6], uvs[7]],
                        color,
                    });
                    vertices.push(Vertex {
                        position: [positions[0], positions[1]],
                        uv: [uvs[0], uvs[1]],
                        color,
                    });

                    gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));

                    gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, as_u8_slice(&vertices));
                    gl.draw_arrays(glow::TRIANGLES, 0, vertices.len() as i32);

                    vertices.clear();
                }
                spine::enums::AttachmentType::Mesh => {}
                _ => println!("Ignoring attachment"),
            }
        }
    }
}

fn main() {
    unsafe {
        // Create a context from a WebGL2 context on wasm32 targets
        #[cfg(all(target_arch = "wasm32", feature = "web-sys"))]
        let (_window, gl, _events_loop, render_loop, shader_version) = {
            use wasm_bindgen::JsCast;
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            let webgl2_context = canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::WebGl2RenderingContext>()
                .unwrap();
            (
                (),
                glow::Context::from_webgl2_context(webgl2_context),
                (),
                glow::RenderLoop::from_request_animation_frame(),
                "#version 300 es",
            )
        };

        // Create a context from a glutin window on non-wasm32 targets
        #[cfg(feature = "window-glutin")]
        let (gl, event_loop, windowed_context, shader_version) = {
            let el = glutin::event_loop::EventLoop::new();
            let wb = glutin::window::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
            let windowed_context = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(wb, &el)
                .unwrap();
            let windowed_context = windowed_context.make_current().unwrap();
            let context = glow::Context::from_loader_function(|s| {
                windowed_context.get_proc_address(s) as *const _
            });
            (context, el, windowed_context, "#version 410")
        };

        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#" layout (location = 0) in vec2 a_position;
                layout (location = 1) in vec2 a_texCoords;
                layout (location = 2) in vec4 a_color;

                uniform mat4 view_proj;

                out vec2 v_texCoord;
                out vec4 v_color;

                void main() {
                    v_color = a_color;
                    v_texCoord = a_texCoords;
                    gl_Position = view_proj * vec4(a_position, 0.0, 1.0);
                }"#,
            r#"
                uniform sampler2D texture;

                in vec2 v_texCoord;
                in vec4 v_color;
                
                out vec4 fragColor;

                void main() {
                    vec4 texColor = texture2D(texture, v_texCoord);
                    fragColor = texColor * v_color;
                }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!(gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!(gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.enable(glow::BLEND);

        gl.use_program(Some(program));
        gl.clear_color(0.1, 0.2, 0.3, 1.0);

        // Load the skeleton stuff and create our buffers and all that jazz
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));

        let vbo = gl.create_buffer().unwrap();
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_size(
            glow::ARRAY_BUFFER,
            (2048 * std::mem::size_of::<Vertex>()) as i32,
            glow::DYNAMIC_DRAW,
        );

        gl.vertex_attrib_pointer_f32(
            0,
            2,
            glow::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            0,
        );
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(
            1,
            2,
            glow::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            8,
        );
        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(
            2,
            4,
            glow::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            16,
        );
        gl.enable_vertex_attrib_array(2);

        let view_proj_array = [
            0.001_757_812_5,
            0.0,
            0.0,
            0.0,
            0.0,
            0.002_343_75,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.8,
            0.0,
            0.0,
            0.0,
            -1.0,
            1.0,
        ];

        gl.uniform_matrix_4_f32_slice(
            gl.get_uniform_location(program, "view_proj"),
            false,
            &view_proj_array,
        );

        gl.active_texture(glow::TEXTURE0);
        gl.uniform_1_i32(gl.get_uniform_location(program, "texture"), 0);

        let test_case = &TEST_CASES[0];
        let atlas = Atlas::from_file(test_case.atlas(), |atlas_page, path| {
            // Load the image
            let img_src = image::open(path).unwrap();

            if atlas_page.format() != spine::enums::AtlasFormat::RGBA8888
                || img_src.color() != image::ColorType::Rgba8
            {
                panic!("Unsupported image format");
            }
            let img = img_src.as_rgba8().unwrap();

            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                atlas_page.wrap().0.glow() as i32,
            ); // set texture wrapping to gl::REPEAT (default wrapping method)
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                atlas_page.wrap().1.glow() as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                atlas_page.min_filter().glow() as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                atlas_page.mag_filter().glow() as i32,
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                img.width() as i32,
                img.height() as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(img.as_flat_samples().as_slice()),
            );

            println!("Prepared texture: id = {}, {}", texture, path.display());

            texture
        })
        .unwrap();

        let skeleton_data = SkeletonData::from_binary_file(test_case.binary(), atlas).unwrap();
        let animation_data = AnimationStateData::new(&skeleton_data);

        let mut skeleton = Skeleton::new(&skeleton_data);
        let mut animation = AnimationState::new(&animation_data);

        let animations = skeleton_data.animations();
        animations.iter().for_each(|a| {
            println!("animation: {}", a.name());
        });
        animation.set_by_name(animations[0].name(), TrackIndex::zero(), true);

        #[cfg(feature = "window-glutin")]
        {
            use glutin::event::{Event, WindowEvent};
            use glutin::event_loop::ControlFlow;

            let mut last = Instant::now();
            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Poll;

                match event {
                    Event::MainEventsCleared => {
                        windowed_context.window().request_redraw();
                    }
                    Event::RedrawRequested(_) => {
                        gl.clear(glow::COLOR_BUFFER_BIT);

                        let now = Instant::now();
                        let delta = now - last;
                        last = now;

                        draw_skeleton(&mut skeleton, &mut animation, &gl, delta.as_secs_f32());
                        windowed_context.swap_buffers().unwrap();
                    }
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            windowed_context.resize(*physical_size);
                        }
                        WindowEvent::CloseRequested => {
                            gl.delete_program(program);
                            gl.delete_buffer(vbo);
                            gl.delete_vertex_array(vao);

                            *control_flow = ControlFlow::Exit
                        }
                        _ => (),
                    },
                    _ => (),
                }
            });
        }

        /*#[cfg(not(feature = "window-glutin"))]
        render_loop.run(move |running: &mut bool| {
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);

            if !*running {
                gl.delete_program(program);
                gl.delete_buffer(vbo);
            }
        });
        */
    }
}
