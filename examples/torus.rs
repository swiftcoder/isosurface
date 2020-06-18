// Copyright 2018 Tristam MacDonald
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

extern crate cgmath;
#[macro_use]
extern crate glium;
extern crate glium_text_rusttype as glium_text;
extern crate isosurface;

mod common;

use crate::common::reinterpret_cast_slice;
use crate::common::sources::{CubeSphere, Torus};
use crate::common::text::layout_text;
use cgmath::{vec3, Matrix4, Point3};
use glium::backend::Facade;
use glium::draw_parameters::PolygonMode;
use glium::glutin;
use glium::glutin::{
    dpi::LogicalSize,
    event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
    Api, GlProfile, GlRequest,
};
use glium::index::PrimitiveType;
use glium::Surface;
use isosurface::linear_hashed_marching_cubes::LinearHashedMarchingCubes;
use isosurface::marching_cubes::MarchingCubes;
use isosurface::source::CentralDifference;

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

implement_vertex!(Vertex, position, normal);

const HELP_TEXT: &'static str =
    "Press [A] to switch Algorithm, [S] to switch Shape, or [W] to toggle Wireframe";

struct GenerateResult(glium::VertexBuffer<Vertex>, glium::IndexBuffer<u32>, String);

fn generate<F>(display: &F, shape: usize, algorithm: usize) -> GenerateResult
where
    F: Facade,
{
    let mut vertices = vec![];
    let mut indices = vec![];

    let torus = CentralDifference::new(Torus {});
    let cube_sphere = CentralDifference::new(CubeSphere {});

    let shape_name = match shape % 2 {
        0 => "Torus",
        _ => "Cube Sphere",
    };

    let algorithm_name = match algorithm % 2 {
        0 => {
            let mut marching_cubes = MarchingCubes::new(128);
            match shape % 2 {
                0 => marching_cubes.extract_with_normals(&torus, &mut vertices, &mut indices),
                _ => marching_cubes.extract_with_normals(&cube_sphere, &mut vertices, &mut indices),
            }
            "Marching Cubes"
        }
        _ => {
            let mut linear_hashed_marching_cubes = LinearHashedMarchingCubes::new(7);
            match shape % 2 {
                0 => linear_hashed_marching_cubes.extract_with_normals(
                    &torus,
                    &mut vertices,
                    &mut indices,
                ),
                _ => linear_hashed_marching_cubes.extract_with_normals(
                    &cube_sphere,
                    &mut vertices,
                    &mut indices,
                ),
            }
            "Linear Hashed Marching Cubes"
        }
    };

    let vertex_buffer: glium::VertexBuffer<Vertex> =
        glium::VertexBuffer::new(display, reinterpret_cast_slice(&vertices))
            .expect("failed to create vertex buffer");

    let index_buffer: glium::IndexBuffer<u32> =
        glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices)
            .expect("failed to create index buffer");

    GenerateResult(
        vertex_buffer,
        index_buffer,
        format!(
            "{} - {}. {} vertices {} triangles",
            shape_name,
            algorithm_name,
            vertices.len() / 6,
            indices.len() / 3
        ),
    )
}

fn main() {
    let events_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title("torus")
        .with_inner_size(LogicalSize {
            width: 1024.0,
            height: 768.0,
        });
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_depth_buffer(24);
    let display =
        glium::Display::new(window, context, &events_loop).expect("failed to create display");

    let text_system = glium_text::TextSystem::new(&display);
    let font = glium_text::FontTexture::new(
        &display,
        &include_bytes!("fonts/RobotoMono-Regular.ttf")[..],
        24,
        glium_text::FontTexture::ascii_character_list(),
    )
    .unwrap();

    let mut wireframe = false;
    let mut shape = 0;
    let mut algorithm = 0;

    let mut generated = generate(&display, shape, algorithm);

    let program = program!(&display,
        330 => {
            vertex: "#version 330
                    uniform mat4 model_view_projection;

                    layout(location=0) in vec3 position;
                    layout(location=1) in vec3 normal;

                    out vec3 vNormal;

                    void main() {
                        gl_Position = model_view_projection * vec4(position, 1.0);
                        vNormal = normal;
                    }
                ",
            fragment: "#version 330
                    in vec3 vNormal;

                    layout(location=0) out vec4 color;

                    vec3 hemisphere(vec3 normal) {
                        const vec3 light = vec3(0.1, -1.0, 0.0);
                        float NdotL = dot(normal, light)*0.5 + 0.5;
                        return mix(vec3(0.886, 0.757, 0.337), vec3(0.518, 0.169, 0.0), NdotL);
                    }

                    void main() {
                        color = vec4(hemisphere(normalize(vNormal)), 1.0);
                    }
                "
        },
    )
    .expect("failed to compile shaders");

    let (view_w, view_h) = display.get_framebuffer_dimensions();
    let aspect = view_w as f32 / view_h as f32;
    let projection = cgmath::perspective(cgmath::Deg(45.0), aspect, 0.01, 1000.0);
    let view = Matrix4::look_at(
        Point3::new(-0.25, -0.25, -0.25),
        Point3::new(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
    );

    let help_transform = layout_text(50.0, aspect, 1.0, 1.0);
    let label_transform = layout_text(50.0, aspect, 1.0, 50.0 / aspect - 2.0);

    events_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode,
                            ..
                        },
                    ..
                } => match virtual_keycode {
                    Some(VirtualKeyCode::Escape) => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit
                    }
                    Some(VirtualKeyCode::W) => {
                        wireframe = !wireframe;
                    }
                    Some(VirtualKeyCode::A) => {
                        algorithm += 1;
                        generated = generate(&display, shape, algorithm);
                    }
                    Some(VirtualKeyCode::S) => {
                        shape += 1;
                        generated = generate(&display, shape, algorithm);
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }

        let mut surface = display.draw();
        surface.clear_color_and_depth((0.024, 0.184, 0.337, 0.0), 1.0);

        let uniforms = uniform! {
            model_view_projection: Into::<[[f32; 4]; 4]>::into(projection * view),
        };

        let polygon_mode = if wireframe {
            PolygonMode::Line
        } else {
            PolygonMode::Fill
        };

        let draw_parameters = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            point_size: Some(8.0),
            polygon_mode,
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        surface
            .draw(
                &generated.0,
                &generated.1,
                &program,
                &uniforms,
                &draw_parameters,
            )
            .expect("failed to draw to surface");

        let mut text = glium_text::TextDisplay::new(&text_system, &font, &generated.2);
        glium_text::draw(
            &text,
            &text_system,
            &mut surface,
            Into::<[[f32; 4]; 4]>::into(label_transform),
            (1.0, 1.0, 1.0, 1.0),
        )
        .expect("failed to render text");

        text.set_text(HELP_TEXT);
        glium_text::draw(
            &text,
            &text_system,
            &mut surface,
            Into::<[[f32; 4]; 4]>::into(help_transform),
            (1.0, 1.0, 1.0, 1.0),
        )
        .expect("failed to render text");

        surface.finish().expect("failed to finish rendering frame");
    });
}
