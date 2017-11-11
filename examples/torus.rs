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
use glium::index::PrimitiveType;
use glutin::{GlProfile, GlRequest, Api, Event, WindowEvent, ControlFlow};
use cgmath::{vec3, Matrix4, Point3};
use isosurface::marching_cubes::MarchingCubes;
use isosurface::source::CentralDifference;
use common::sources::Torus;
use common::reinterpret_cast_slice;

#[derive(Copy, Clone)]
#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

implement_vertex!(Vertex, position, normal);

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("torus")
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_gl_profile(GlProfile::Core)
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_depth_buffer(24);
    let display = glium::Display::new(window, context, &events_loop)
        .expect("failed to create display");

    let torus = Torus{};
    let central_difference = CentralDifference::new(Box::new(torus));

    let mut vertices = vec![];
    let mut indices = vec![];
    let mut marching_cubes = MarchingCubes::new(256);

    marching_cubes.extract_with_normals(&central_difference, &mut vertices, &mut indices);

    let vertex_buffer: glium::VertexBuffer<Vertex> = {
        glium::VertexBuffer::new(
            &display,
            reinterpret_cast_slice(&vertices)
        ).expect("failed to create vertex buffer")
    };

    let index_buffer: glium::IndexBuffer<u32> =
        glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &indices)
            .expect("failed to create index buffer");

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
        ).expect("failed to compile shaders");

    let projection = cgmath::perspective(cgmath::Deg(45.0), 1024.0/768.0, 0.01, 1000.0);
    let view = Matrix4::look_at(Point3::new(-0.25, -0.25, -0.25), Point3::new(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));

    events_loop.run_forever(|event| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Closed => return ControlFlow::Break,
                _ => (),
            },
            _ => (),
        }

        let mut surface = display.draw();
        surface.clear_color_and_depth((0.024, 0.184, 0.337, 0.0), 1.0);

        let uniforms = uniform! {
            model_view_projection: Into::<[[f32; 4]; 4]>::into(projection * view),
        };

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
            ..Default::default()
        };

        surface.draw(
            &vertex_buffer,
            &index_buffer,
            &program,
            &uniforms,
            &params,
        ).expect("failed to draw to surface");

        surface.finish().expect("failed to finish rendering frame");

        ControlFlow::Continue
    });

}