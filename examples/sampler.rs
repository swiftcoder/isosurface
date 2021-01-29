// Copyright 2021 Tristam MacDonald
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

use crate::{
    common::reinterpret_cast_slice, common::sources::DemoSource, common::text::layout_text,
};
use cgmath::{vec3, Matrix4, Point3};
use glium::index::PrimitiveType;
use glium::Surface;
use glium::{
    backend::Facade,
    draw_parameters::PolygonMode,
    glutin::{
        self,
        dpi::LogicalSize,
        event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent},
        Api, GlProfile, GlRequest,
    },
};
use isosurface::{
    distance::Signed,
    extractor::IndexedInterleavedNormals,
    feature::ParticleBasedMinimisation,
    implicit::{Cylinder, Difference, Intersection, RectangularPrism, Sphere, Torus, Union},
    math::Vec3,
    sampler::Sampler,
    source::CentralDifference,
    DualContouring, ExtendedMarchingCubes, LinearHashedMarchingCubes, MarchingCubes,
};

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

implement_vertex!(Vertex, position, normal);

const HELP_TEXT: &'static str =
    "Press [A] to change Algorithm, [S] Shape, [C] Complexity, [N] Normals, or [W] Wireframe";

struct GenerateResult(glium::VertexBuffer<Vertex>, glium::IndexBuffer<u32>, String);

fn generate<F>(display: &F, shape: usize, algorithm: usize, complexity: usize) -> GenerateResult
where
    F: Facade,
{
    let mut vertices = vec![];
    let mut indices = vec![];

    let sources = [
        (DemoSource::new(Torus::new(0.25, 0.1)), "Torus"),
        (DemoSource::new(Sphere::new(0.3)), "Sphere"),
        (
            DemoSource::new(RectangularPrism::new(Vec3::from_scalar(0.2))),
            "Box",
        ),
        (DemoSource::new(Cylinder::new(0.25, 0.2)), "Cylinder"),
        (
            DemoSource::new(CentralDifference::new(Union::new(
                Difference::new(
                    Sphere::new(0.25),
                    RectangularPrism::new(Vec3::from_scalar(0.2)),
                ),
                Cylinder::new(0.02, 0.25),
            ))),
            "Sphere subtracted from Cube",
        ),
        (
            DemoSource::new(CentralDifference::new(Intersection::new(
                Sphere::new(0.3),
                RectangularPrism::new(Vec3::from_scalar(0.2)),
            ))),
            "Sphere intersected with Cube",
        ),
    ];

    let (source, shape_name) = &sources[shape % sources.len()];
    let sampler = Sampler::new(source);

    let max_level = 3 + complexity % 5;
    let grid_size = 2usize.pow(max_level as u32);

    let mut extractor = IndexedInterleavedNormals::new(&mut vertices, &mut indices, &sampler);

    let algorithm_name = match algorithm % 4 {
        0 => {
            let mut marching_cubes = MarchingCubes::<Signed>::new(grid_size);
            marching_cubes.extract(&sampler, &mut extractor);
            "Marching Cubes"
        }
        1 => {
            let mut linear_hashed_marching_cubes = LinearHashedMarchingCubes::new(max_level);
            linear_hashed_marching_cubes.extract(&sampler, &mut extractor);
            "Linear Hashed Marching Cubes"
        }
        2 => {
            let mut extended_marching_cubes = ExtendedMarchingCubes::new(grid_size);
            extended_marching_cubes.extract(&sampler, &mut extractor);
            "Extended Marching Cubes"
        }
        _ => {
            let mut dual_contouring = DualContouring::new(grid_size, ParticleBasedMinimisation {});
            dual_contouring.extract(&sampler, &mut extractor);
            "Dual Contouring"
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
            "{} - {} with {} octree levels, {} grid size, {} vertices, {} triangles",
            shape_name,
            algorithm_name,
            max_level,
            grid_size,
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
    let mut complexity = 4;
    let mut show_normals = false;

    let mut generated = generate(&display, shape, algorithm, complexity);

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
                    uniform float show_normals;

                    in vec3 vNormal;

                    layout(location=0) out vec4 color;

                    vec3 hemisphere(vec3 normal) {
                        const vec3 light = vec3(0.1, -1.0, 0.0);
                        float NdotL = dot(normal, light)*0.5 + 0.5;
                        return mix(vec3(0.3605, 0.2176, 0.005), vec3(0.7381, 0.531, 0.1003), NdotL);
                    }

                    void main() {
                        if (show_normals > 0.5) {
                            color = vec4(normalize(vNormal)*0.5 + 0.5, 1.0);
                        } else {
                            color = vec4(hemisphere(normalize(vNormal)), 1.0);
                        }
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

    let help_transform = layout_text(65.0, aspect, 1.0, 1.0);
    let label_transform = layout_text(65.0, aspect, 1.0, 65.0 / aspect - 2.0);

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
                        generated = generate(&display, shape, algorithm, complexity);
                    }
                    Some(VirtualKeyCode::S) => {
                        shape += 1;
                        generated = generate(&display, shape, algorithm, complexity);
                    }
                    Some(VirtualKeyCode::C) => {
                        complexity += 1;
                        generated = generate(&display, shape, algorithm, complexity);
                    }
                    Some(VirtualKeyCode::N) => {
                        show_normals = !show_normals;
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }

        let mut surface = display.draw();
        surface.clear_color_and_depth((0.011, 0.0089, 0.1622, 0.0), 1.0);

        let uniforms = uniform! {
            model_view_projection: Into::<[[f32; 4]; 4]>::into(projection * view),
            show_normals: if show_normals {1.0f32} else {0.0},
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
