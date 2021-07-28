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
    in vec3 color;
    out vec3 in_color;

    uniform mat4 matrix;

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
    color: [f64; 3],
}

implement_vertex!(Vertex, position, color);

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

        for person in sim.people() {
            let mut shape = vec![];
            let n_sides = 20;
            for i in 0..n_sides {
                let ang = 2.0 * 3.14159 * (i as f64) / n_sides as f64;
                shape.push(Vertex {
                    position: [
                        person.pos().x + RADIUS * ang.cos(),
                        person.pos().y + RADIUS * ang.sin(),
                    ],
                    color: color(person.status()),
                });
            }

            let vertex_buffer = VertexBuffer::new(display, &shape).unwrap();
            let indices = index::NoIndices(index::PrimitiveType::TriangleFan);
            let uniforms = uniform! {
                matrix: matrix.inner(),
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

fn color(status: &Status) -> [f64; 3] {
    if status.infected().is_some() {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 0.8, 0.0]
    }
}
