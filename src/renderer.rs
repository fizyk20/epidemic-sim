mod matrix;

use glium::{implement_vertex, index, uniform, Display, Program, Surface, VertexBuffer};
use nalgebra::Vector2;

use crate::simulation::{
    person::{Status, RADIUS},
    Simulation,
};

use matrix::Matrix;

const VERTEX_SHADER_SRC: &'static str = r#"
    #version 140

    in vec2 position;

    uniform mat4 matrix;
    uniform vec3 color;
    out vec3 in_color;

    void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
        in_color = color;
    }
"#;

const FRAGMENT_SHADER_SRC: &'static str = r#"
    #version 140

    in vec3 in_color;
    out vec4 color;

    void main() {
        color = vec4(in_color, 1.0);
    }
"#;

#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f64; 2],
}

implement_vertex!(Vertex, position);

pub struct Renderer {
    center: Vector2<f64>,
    size_smaller: f64,
    program: Program,
}

impl Renderer {
    pub fn new(display: &Display, center_x: f64, center_y: f64, size_smaller: f64) -> Self {
        Renderer {
            center: Vector2::new(center_x, center_y),
            size_smaller,
            program: Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap(),
        }
    }

    fn circle(display: &Display) -> VertexBuffer<Vertex> {
        let mut shape = vec![];
        let n_sides = 20;
        for i in 0..n_sides {
            let ang = 2.0 * 3.14159 * (i as f64) / n_sides as f64;
            shape.push(Vertex {
                position: [RADIUS * ang.cos(), RADIUS * ang.sin()],
            });
        }
        VertexBuffer::new(display, &shape).unwrap()
    }

    pub fn draw(&self, display: &Display, sim: &Simulation) {
        let mut target = display.draw();

        target.clear_color(1.0, 1.0, 1.0, 1.0);

        let (size_x, size_y) = target.get_dimensions();
        let (size_x, size_y) = (size_x as f64, size_y as f64);

        let matrix = Matrix::translation(-self.center.x as f32, -self.center.y as f32)
            * if size_x < size_y {
                Matrix::scale(
                    2.0 / self.size_smaller as f32,
                    (2.0 * size_x / size_y / self.size_smaller) as f32,
                )
            } else {
                Matrix::scale(
                    (2.0 * size_y / size_x / self.size_smaller) as f32,
                    2.0 / self.size_smaller as f32,
                )
            };

        let vertex_buffer = Self::circle(display);
        let indices = index::NoIndices(index::PrimitiveType::TriangleFan);

        for person in sim.people() {
            let matrix2 =
                Matrix::translation(person.pos().x as f32, person.pos().y as f32) * matrix;
            let uniforms = uniform! {
                matrix: matrix2.inner(),
                color: color(person.status()),
            };

            target
                .draw(
                    &vertex_buffer,
                    &indices,
                    &self.program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
        }

        target.finish().unwrap();
    }
}

fn color(status: &Status) -> [f32; 3] {
    if status.infected().is_some() {
        if status.vaccinated() {
            [0.7, 0.0, 0.7]
        } else {
            [1.0, 0.0, 0.0]
        }
    } else {
        if status.vaccinated() {
            [0.0, 0.0, 1.0]
        } else if status.past_infected() {
            [0.5, 0.5, 0.0]
        } else {
            [0.0, 0.7, 0.0]
        }
    }
}
