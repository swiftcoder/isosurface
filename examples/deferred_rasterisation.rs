// Copyright 2017 Tristam MacDonald
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate isosurface;

mod common;

use glium::glutin;
use glium::Surface;
use glium::texture::{MipmapsOption, Texture2d, DepthTexture2d, UncompressedFloatFormat, DepthFormat};
use glutin::{GlProfile, GlRequest, Api, Event, WindowEvent, ControlFlow};
use cgmath::{vec3, Matrix4, Point3, SquareMatrix};
use isosurface::point_cloud::PointCloud;
use isosurface::source::CentralDifference;
use common::sources::Torus;
use common::reinterpret_cast_slice;

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    position : [f32; 3],
}

implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
#[repr(C)]
struct VertexWithNormal {
    position : [f32; 3],
    normal : [f32; 3],
}

implement_vertex!(VertexWithNormal, position, normal);

// This technique is derived from an image tweeted by Gavan Woolery (gavanw@). it needs some
// refinement, but I think I've captured an approximation of his rendering technique.
// https://twitter.com/gavanw/status/717265068086308865

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("deferred rasterisation")
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop)
        .expect("failed to create display");

    let (width, height) = display.gl_window().get_inner_size_pixels().unwrap();

    let subdivisions = 64;

    let torus = Torus{};
    let central_difference = CentralDifference::new(Box::new(torus));

    let mut vertices = vec![];
    let mut marcher = PointCloud::new(subdivisions);

    marcher.extract_midpoints_with_normals(&central_difference, &mut vertices);

    let vertex_buffer: glium::VertexBuffer<VertexWithNormal> = {
        glium::VertexBuffer::new(
            &display,
            reinterpret_cast_slice(&vertices)
        ).expect("failed to create vertex buffer")
    };

    let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::Points);

    let program = program!(&display,
            330 => {
                vertex: "#version 330
                    uniform mat4 model_view_projection;
                    uniform mat4 model;

                    layout(location=0) in vec3 position;
                    layout(location=1) in vec3 normal;

                    out vec3 vPosition;
                    out vec3 vNormal;

                    void main() {
                        vPosition = (model * vec4(position, 1.0)).xyz;
                        vNormal = (model * vec4(normal, 0.0)).xyz;
                        gl_Position = model_view_projection * vec4(position, 1.0);
                    }
                ",
                fragment: "#version 330
                    in vec3 vPosition;
                    in vec3 vNormal;

                    layout(location=0) out vec4 color;
                    layout(location=1) out vec4 normal;

                    void main() {
                        color = vec4(vPosition, 1.0);
                        normal = vec4(normalize(vNormal), 0.0);
                    }
                "
            },
        ).expect("failed to compile shaders");

    let program2 = program!(&display,
            330 => {
                vertex: "#version 330
                    layout(location=0) in vec3 position;

                    out vec3 vPosition;

                    void main() {
                        vPosition = position;
                        gl_Position = vec4(position, 1.0);
                    }
                ",
                fragment: "#version 330
                    uniform sampler2D main_texture;
                    uniform sampler2D main_normal;
                    uniform vec2 direction;
                    uniform float voxel_size;
                    uniform vec2 pixel_dims;
                    uniform mat4 view_projection_inverse;
                    uniform bool last;

                    layout(location=0) out vec4 color;
                    layout(location=1) out vec4 normal;

                    const int taps = 16;
                    const vec3 one_vec = vec3(1.0, 1.0, 1.0);

                    in vec3 vPosition;

                    vec2 aabbIntersect(vec3 rayOrig, vec3 rayDir, vec3 minv, vec3 maxv) {
                        vec3 invR = 1.0 / rayDir;
                        vec3 tbot = invR * (minv-rayOrig);
                        vec3 ttop = invR * (maxv-rayOrig);
                        vec3 tmin = min(ttop, tbot);
                        vec3 tmax = max(ttop, tbot);
                        vec2 t = max(tmin.xx, tmin.yz);
                        float t0 = max(t.x, t.y);
                        t = min(tmax.xx, tmax.yz);
                        float t1 = min(t.x, t.y);
                        return vec2(t0,t1); // if (t0 <= t1) { did hit } else { did not hit }
                    }

                    vec3 hemisphere(vec3 normal) {
                        const vec3 light = vec3(0.1, -1.0, 0.0);
                        float NdotL = dot(normal, light)*0.5 + 0.5;
                        return mix(vec3(0.886, 0.757, 0.337), vec3(0.518, 0.169, 0.0), NdotL);
                    }

                    void main() {
                        vec4 eye = view_projection_inverse * vec4(0.0, 0.0, -1.0, 1.0);
                        eye.xyz /= eye.w;
                        vec4 screen = view_projection_inverse * vec4(vPosition, 1.0);
                        screen.xyz /= screen.w;
                        vec3 eye_dir = normalize(screen.xyz - eye.xyz);

                        vec2 vTexcoord = vPosition.xy*0.5 + 0.5;

                        vec3 result = vec3(0.0, 0.0, 0.0);
                        vec3 result_normal = vec3(0.0, 0.0, 0.0);
                        float best = 9999999.0;

                        for (int i = -taps; i <= taps; ++i) {
                            vec2 coord = vTexcoord + vec2(i)*direction*pixel_dims;
                            vec3 p = texture(main_texture, coord).xyz;
                            if (dot(abs(p), one_vec) > 0.0) {
                                vec2 box = aabbIntersect(eye.xyz, eye_dir, p - voxel_size, p + voxel_size);
                                if (box.x <= box.y) {
                                    float distance = box.x;
                                    if (distance <= best) {
                                        best = distance;
                                        result = p;
                                        result_normal = texture(main_normal, coord).xyz;
                                    }
                                }
                            }
                        }

                        if (last && best < 9999999.0) {
                            color = vec4(hemisphere(result_normal), 1.0);
                        } else {
                            color = vec4(result, 0.0);
                            normal = vec4(result_normal, 0.0);
                        }
                    }
                "
            },
        ).expect("failed to compile shaders");

    let projection = cgmath::perspective(cgmath::Deg(45.0), (width as f32)/(height as f32), 0.01, 1000.0);
    let view = Matrix4::look_at(Point3::new(-0.25, -0.25, -0.25), Point3::new(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
    let model = Matrix4::identity();

    // We need two textures to ping-pong between, and one of them needs an attached depth buffer for the initial pass
    let position1 = Texture2d::empty_with_format(&display, UncompressedFloatFormat::F32F32F32F32, MipmapsOption::NoMipmap, width, height).unwrap();
    let normal1 = Texture2d::empty_with_format(&display, UncompressedFloatFormat::F32F32F32F32, MipmapsOption::NoMipmap, width, height).unwrap();
    let depth1 = DepthTexture2d::empty_with_format(&display, DepthFormat::F32, MipmapsOption::NoMipmap, width, height).unwrap();

    let position2 = Texture2d::empty_with_format(&display, UncompressedFloatFormat::F32F32F32F32, MipmapsOption::NoMipmap, width, height).unwrap();
    let normal2 = Texture2d::empty_with_format(&display, UncompressedFloatFormat::F32F32F32F32, MipmapsOption::NoMipmap, width, height).unwrap();

    // This extra scope is needed as a workaround for https://github.com/rust-lang/rust/issues/38915
    {
        let mut framebuffer1 = glium::framebuffer::MultiOutputFrameBuffer::with_depth_buffer(&display, vec![("color", &position1), ("normal", &normal1)], &depth1).unwrap();
        let mut framebuffer2 = glium::framebuffer::MultiOutputFrameBuffer::new(&display, vec![("color", &position2), ("normal", &normal2)]).unwrap();

        let quad_vertex_buffer = {
            glium::VertexBuffer::new(&display,
                                     &[
                                         Vertex { position: [-1.0, -1.0, 1.0] },
                                         Vertex { position: [1.0, -1.0, 1.0] },
                                         Vertex { position: [1.0, 1.0, 1.0] },
                                         Vertex { position: [-1.0, 1.0, 1.0] },
                                     ]
            ).unwrap()
        };

        let quad_index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList,
                                                        &[0u16, 1, 2, 0, 2, 3]).unwrap();

        events_loop.run_forever(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Closed => return ControlFlow::Break,
                    _ => (),
                },
                _ => (),
            }

            // First pass, render depth-tested points into the first buffer
            {
                framebuffer1.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

                let uniforms = uniform! {
                model_view_projection: Into::<[[f32; 4]; 4]>::into(projection * view * model),
                model: Into::<[[f32; 4]; 4]>::into(model),
            };

                let params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    point_size: Some(1.0),
                    ..Default::default()
                };

                framebuffer1.draw(
                    &vertex_buffer,
                    &index_buffer,
                    &program,
                    &uniforms,
                    &params,
                ).expect("failed to draw to surface");
            }

            // pass 1 through N-1, ping-pong render both buffers in turn, spreading the points across
            // the faces of their respective cubes
            for i in 0..3 {
                let framebuffer = if i % 2 == 0 { &mut framebuffer2 } else { &mut framebuffer1 };
                framebuffer.clear_color(0.0, 0.0, 0.0, 0.0);

                let uniforms = uniform! {
                main_texture: (if i % 2 == 0 {&position1} else {&position2}),
                main_normal: (if i % 2 == 0 {&normal1} else {&normal2}),
                direction: [((i+1) % 2) as f32, (i % 2) as f32],
                voxel_size: 0.5 / (subdivisions as f32),
                pixel_dims: [1.0 / (width as f32), 1.0 / (height as f32)],
                view_projection_inverse: Into::<[[f32; 4]; 4]>::into((projection * view).invert().unwrap()),
                last: false,
            };

                framebuffer.draw(
                    &quad_vertex_buffer,
                    &quad_index_buffer,
                    &program2,
                    &uniforms,
                    &Default::default(),
                ).expect("failed to draw to surface");
            }

            // final pass, composite the last buffer to the screen, performing lighting in the process
            {
                let mut surface = display.draw();
                surface.clear_color_and_depth((0.306, 0.267, 0.698, 0.0), 1.0);

                let uniforms = uniform! {
                main_texture: &position2,
                main_normal: &normal2,
                direction: [0f32, 1.0],
                voxel_size: 0.5 / (subdivisions as f32),
                pixel_dims: [1.0 / (width as f32), 1.0 / (height as f32)],
                view_projection_inverse: Into::<[[f32; 4]; 4]>::into((projection * view).invert().unwrap()),
                last: true,
            };

                let params = glium::DrawParameters {
                    blend: glium::Blend::alpha_blending(),
                    ..Default::default()
                };

                surface.draw(
                    &quad_vertex_buffer,
                    &quad_index_buffer,
                    &program2,
                    &uniforms,
                    &params,
                ).expect("failed to draw to surface");

                surface.finish().expect("failed to finish rendering frame");
            }

            ControlFlow::Continue
        });
    }
}
