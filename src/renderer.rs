mod matrix;
mod stats_buf;

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
use stats_buf::StatsBuf;

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
    stats_buf: StatsBuf,
    last_t: f64,
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
            stats_buf: StatsBuf::new(),
            last_t: -0.01,
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

    fn draw_text(
        &self,
        target: &mut Frame,
        text: &str,
        matrix: Matrix,
        draw_parameters: DrawParameters,
    ) {
        let text = TextDisplay::new(&self.text_system, &self.font, text);

        glium_text::draw(
            &text,
            &self.text_system,
            target,
            matrix.inner(),
            (0.0, 0.0, 0.0, 1.0),
            draw_parameters.clone(),
        );
    }

    fn draw_numbers(&self, target: &mut Frame, sim: &Simulation) {
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
                width: (size_x - box_size) / 2 - 20,
                height: size_y - 20,
            }
        } else {
            Rect {
                left: 10,
                bottom: 0,
                width: size_x / 2 - 20,
                height: size_y - box_size - 20,
            }
        };

        let w = viewport.width as f32;
        let h = viewport.height as f32;

        let matrix = Matrix::scale(1.0 / 30.0, 1.0 / 30.0)
            * Matrix::translation(-0.5, 0.5 * h / w)
            * Matrix::scale(2.0, 2.0 * w / h);

        let draw_parameters = DrawParameters {
            viewport: Some(viewport),
            ..Default::default()
        };

        let stats = sim.stats();

        self.draw_text(
            target,
            &format!("Population: {}", stats.population),
            Matrix::translation(0.1, -1.0) * matrix,
            draw_parameters.clone(),
        );

        self.draw_text(
            target,
            &format!("Infected: {}", stats.infected),
            Matrix::translation(0.1, -2.5) * matrix,
            draw_parameters.clone(),
        );

        self.draw_text(
            target,
            &format!("   of these, vaccinated: {}", stats.vaccinated_infected),
            Matrix::translation(0.1, -4.0) * matrix,
            draw_parameters.clone(),
        );

        self.draw_text(
            target,
            &format!("Healed: {}", stats.healed),
            Matrix::translation(0.1, -5.5) * matrix,
            draw_parameters.clone(),
        );

        self.draw_text(
            target,
            &format!("Vaccinated: {}", stats.vaccinated),
            Matrix::translation(0.1, -7.0) * matrix,
            draw_parameters.clone(),
        );

        self.draw_text(
            target,
            &format!("Dead: {}", stats.dead),
            Matrix::translation(0.1, -8.5) * matrix,
            draw_parameters,
        );
    }

    fn draw_key(&self, display: &Display, target: &mut Frame) {
        let (size_x, size_y) = target.get_dimensions();

        let (box_size, horizontal) = if size_x < size_y {
            (size_x, false)
        } else {
            (size_y, true)
        };

        let viewport = if horizontal {
            Rect {
                left: box_size + (size_x - box_size) / 2 + 10,
                bottom: 0,
                width: (size_x - box_size) / 2 - 20,
                height: size_y - 20,
            }
        } else {
            Rect {
                left: 10 + size_x / 2,
                bottom: 0,
                width: size_x / 2 - 20,
                height: size_y - box_size - 20,
            }
        };

        let w = viewport.width as f32;
        let h = viewport.height as f32;

        let matrix = Matrix::scale(1.0 / 30.0, 1.0 / 30.0)
            * Matrix::translation(-0.5, 0.5 * h / w)
            * Matrix::scale(2.0, 2.0 * w / h);

        let draw_parameters = DrawParameters {
            viewport: Some(viewport),
            ..Default::default()
        };

        let elements = [
            (COLOR_HEALTHY, "Healthy"),
            (COLOR_INFECTED, "Infected"),
            (COLOR_HEALED, "Healed"),
            (COLOR_VACCINATED, "Vaccinated"),
            (COLOR_VACCINATED_INFECTED, "Vaccinated and infected"),
            (COLOR_DEAD, "Dead (graph only)"),
        ];

        self.draw_text(
            target,
            "Color key:",
            Matrix::translation(0.1, -1.0) * matrix,
            draw_parameters.clone(),
        );

        let vertex_buffer = Self::circle(display);
        let indices = index::NoIndices(index::PrimitiveType::TriangleFan);

        for (i, (color, name)) in elements.iter().enumerate() {
            let matrix2 = Matrix::translation(0.55, -2.5 - i as f32 * 1.5) * matrix;
            let uniforms = uniform! {
                matrix: matrix2.inner(),
                color: *color,
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

            self.draw_text(
                target,
                name,
                Matrix::translation(1.6, -3.02 - i as f32 * 1.5) * matrix,
                draw_parameters.clone(),
            );
        }
    }

    fn graph_viewport(&self, target: &Frame) -> Rect {
        let (size_x, size_y) = target.get_dimensions();

        let (box_size, horizontal) = if size_x < size_y {
            (size_x, false)
        } else {
            (size_y, true)
        };

        if horizontal {
            Rect {
                left: box_size + 10,
                bottom: size_y / 3,
                width: size_x - box_size - 20,
                height: size_y / 3,
            }
        } else {
            let free_height = size_y - box_size;
            Rect {
                left: 10,
                bottom: free_height / 3,
                width: size_x - 20,
                height: free_height / 3,
            }
        }
    }

    pub fn draw(&mut self, display: &Display, sim: &Simulation) {
        let mut target = display.draw();

        target.clear_color(1.0, 1.0, 1.0, 1.0);

        self.draw_sim(display, &mut target, sim);

        self.draw_numbers(&mut target, sim);

        self.draw_key(display, &mut target);

        if self.last_t < sim.time().floor() {
            self.stats_buf.record(sim.time().floor(), sim.stats());
        }

        self.last_t = sim.time();

        let graph_viewport = self.graph_viewport(&target);
        let draw_parameters = DrawParameters {
            viewport: Some(graph_viewport),
            ..Default::default()
        };
        self.stats_buf
            .draw(display, &mut target, &self, &draw_parameters);

        target.finish().unwrap();
    }
}

const COLOR_HEALTHY: [f32; 3] = [0.0, 0.7, 0.0];
const COLOR_INFECTED: [f32; 3] = [1.0, 0.0, 0.0];
const COLOR_HEALED: [f32; 3] = [0.5, 0.5, 0.0];
const COLOR_VACCINATED: [f32; 3] = [0.0, 0.0, 1.0];
const COLOR_VACCINATED_INFECTED: [f32; 3] = [0.7, 0.0, 0.7];
const COLOR_DEAD: [f32; 3] = [0.2, 0.2, 0.2];

fn color(status: &Status) -> [f32; 3] {
    if status.infected().is_some() {
        if status.vaccinated() {
            COLOR_VACCINATED_INFECTED
        } else {
            COLOR_INFECTED
        }
    } else {
        if status.vaccinated() {
            COLOR_VACCINATED
        } else if status.past_infected() {
            COLOR_HEALED
        } else {
            COLOR_HEALTHY
        }
    }
}
