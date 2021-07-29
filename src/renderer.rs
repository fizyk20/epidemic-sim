mod matrix;

use std::fs::File;

use glium::{
    draw_parameters::DrawParameters, implement_vertex, index, uniform, Display, Frame, Program,
    Rect, Surface, VertexBuffer,
};
use glium_text::{FontTexture, TextDisplay, TextSystem};
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
    text_system: TextSystem,
    font: FontTexture,
}

impl Renderer {
    pub fn new(display: &Display, center_x: f64, center_y: f64, size_smaller: f64) -> Self {
        let text_system = TextSystem::new(display);
        let font = FontTexture::new(display, File::open("DejaVuSans.ttf").unwrap(), 24).unwrap();

        Renderer {
            center: Vector2::new(center_x, center_y),
            size_smaller,
            program: Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap(),
            text_system,
            font,
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

    fn draw_sim(&self, display: &Display, target: &mut Frame, sim: &Simulation) {
        let (size_x, size_y) = target.get_dimensions();

        let (box_size, horizontal) = if size_x < size_y {
            (size_x - 20, false)
        } else {
            (size_y - 20, true)
        };

        let matrix = Matrix::translation(-self.center.x as f32, -self.center.y as f32)
            * Matrix::scale(
                2.0 / self.size_smaller as f32,
                2.0 / self.size_smaller as f32,
            );

        let vertex_buffer = Self::circle(display);
        let indices = index::NoIndices(index::PrimitiveType::TriangleFan);
        let draw_parameters = DrawParameters {
            viewport: Some(Rect {
                left: 10,
                bottom: if horizontal {
                    10
                } else {
                    size_y - box_size - 10
                },
                width: box_size,
                height: box_size,
            }),
            ..Default::default()
        };

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
                    &draw_parameters,
                )
                .unwrap();
        }
    }

    fn draw_numbers(&self, display: &Display, target: &mut Frame, sim: &Simulation) {
        let (size_x, size_y) = target.get_dimensions();

        let (box_size, horizontal) = if size_x < size_y {
            (size_x, false)
        } else {
            (size_y, true)
        };

        let viewport = if horizontal {
            Rect {
                left: box_size + 10,
                bottom: 0,
                width: size_x - box_size - 20,
                height: size_y - 20,
            }
        } else {
            Rect {
                left: 10,
                bottom: 0,
                width: size_x - 20,
                height: size_y - box_size - 20,
            }
        };

        let w = viewport.width as f32;
        let h = viewport.height as f32;

        let matrix = Matrix::scale(1.0 / 60.0, 1.0 / 60.0)
            * Matrix::translation(-0.5, 0.5 * h / w)
            * Matrix::scale(2.0, 2.0 * w / h);

        let draw_parameters = DrawParameters {
            viewport: Some(viewport),
            ..Default::default()
        };

        let stats = sim.stats();

        let population = TextDisplay::new(
            &self.text_system,
            &self.font,
            &format!("Population: {}", stats.population),
        );

        let infected = TextDisplay::new(
            &self.text_system,
            &self.font,
            &format!("Infected: {}", stats.infected),
        );

        let vaccinated_infected = TextDisplay::new(
            &self.text_system,
            &self.font,
            &format!("   of these, vaccinated: {}", stats.vaccinated_infected),
        );

        let healed = TextDisplay::new(
            &self.text_system,
            &self.font,
            &format!("Healed: {}", stats.healed),
        );

        let vaccinated = TextDisplay::new(
            &self.text_system,
            &self.font,
            &format!("Vaccinated: {}", stats.vaccinated),
        );

        let dead = TextDisplay::new(
            &self.text_system,
            &self.font,
            &format!("Dead: {}", stats.dead),
        );

        glium_text::draw(
            &population,
            &self.text_system,
            target,
            (Matrix::translation(0.1, -1.0) * matrix).inner(),
            (0.0, 0.0, 0.0, 1.0),
            draw_parameters.clone(),
        );

        glium_text::draw(
            &infected,
            &self.text_system,
            target,
            (Matrix::translation(0.1, -2.5) * matrix).inner(),
            (0.0, 0.0, 0.0, 1.0),
            draw_parameters.clone(),
        );

        glium_text::draw(
            &vaccinated_infected,
            &self.text_system,
            target,
            (Matrix::translation(0.1, -4.0) * matrix).inner(),
            (0.0, 0.0, 0.0, 1.0),
            draw_parameters.clone(),
        );

        glium_text::draw(
            &healed,
            &self.text_system,
            target,
            (Matrix::translation(0.1, -5.5) * matrix).inner(),
            (0.0, 0.0, 0.0, 1.0),
            draw_parameters.clone(),
        );

        glium_text::draw(
            &vaccinated,
            &self.text_system,
            target,
            (Matrix::translation(0.1, -7.0) * matrix).inner(),
            (0.0, 0.0, 0.0, 1.0),
            draw_parameters.clone(),
        );

        glium_text::draw(
            &dead,
            &self.text_system,
            target,
            (Matrix::translation(0.1, -8.5) * matrix).inner(),
            (0.0, 0.0, 0.0, 1.0),
            draw_parameters,
        );
    }

    pub fn draw(&self, display: &Display, sim: &Simulation) {
        let mut target = display.draw();

        target.clear_color(1.0, 1.0, 1.0, 1.0);

        self.draw_sim(display, &mut target, sim);

        self.draw_numbers(display, &mut target, sim);

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
