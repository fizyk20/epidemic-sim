use glium::{index, uniform, Display, DrawParameters, Frame, IndexBuffer, Surface, VertexBuffer};

use super::{
    matrix::Matrix, Renderer, Vertex, COLOR_DEAD, COLOR_HEALED, COLOR_HEALTHY, COLOR_INFECTED,
    COLOR_VACCINATED, COLOR_VACCINATED_INFECTED,
};

use crate::simulation::Statistics;

pub struct StatsBuf {
    data: Vec<(f64, Statistics)>,
}

impl StatsBuf {
    pub fn new() -> StatsBuf {
        StatsBuf { data: vec![] }
    }

    pub fn min_t(&self) -> f64 {
        self.data.first().map_or(0.0, |(t, _)| *t)
    }

    pub fn max_t(&self) -> f64 {
        self.data.last().map_or(1.0, |(t, _)| *t)
    }

    pub fn record(&mut self, t: f64, stats: Statistics) {
        self.data.push((t, stats));
    }

    fn data_to_vertex(&self, t: f64, num: usize, max_num: usize) -> Vertex {
        let (min_t, max_t) = (self.min_t(), self.max_t());
        let x = (t - min_t) / (max_t - min_t) * 1.8 - 0.8;
        let y = (num as f64) / (max_num as f64) * 1.7 - 0.7;
        Vertex { position: [x, y] }
    }

    pub fn draw(
        &self,
        display: &Display,
        target: &mut Frame,
        renderer: &Renderer,
        draw_parameters: &DrawParameters,
    ) {
        let aspect = draw_parameters
            .viewport
            .map_or(1.0, |rect| rect.width as f32 / rect.height as f32);

        let vertices_axes = vec![
            Vertex {
                position: [-0.8, 1.0],
            },
            Vertex {
                position: [-0.8, -0.7],
            },
            Vertex {
                position: [1.0, -0.7],
            },
        ];

        let vertices_graph = self.generate_graph_vertices();
        let indices = self.generate_graph_indices();
        let colors = [
            COLOR_VACCINATED,
            COLOR_VACCINATED_INFECTED,
            COLOR_INFECTED,
            COLOR_HEALED,
            COLOR_HEALTHY,
            COLOR_DEAD,
        ];

        let vertex_buffer_axes = VertexBuffer::new(display, &vertices_axes).unwrap();
        let index_buffer_axes = index::NoIndices(index::PrimitiveType::LineStrip);

        let vertex_buffer_graph = VertexBuffer::new(display, &vertices_graph).unwrap();

        let matrix = Matrix::identity();

        for i in 0..6 {
            let index_buffer =
                IndexBuffer::new(display, index::PrimitiveType::TriangleStrip, &indices[i])
                    .unwrap();

            // draw dead strip
            let uniforms = uniform! {
                matrix: matrix.inner(),
                color: colors[i],
            };

            target
                .draw(
                    &vertex_buffer_graph,
                    &index_buffer,
                    &renderer.program,
                    &uniforms,
                    &draw_parameters,
                )
                .unwrap();
        }

        // draw axes
        let uniforms = uniform! {
            matrix: matrix.inner(),
            color: [0.0f32, 0.0, 0.0],
        };

        target
            .draw(
                &vertex_buffer_axes,
                &index_buffer_axes,
                &renderer.program,
                &uniforms,
                &draw_parameters,
            )
            .unwrap();

        let text_scale = Matrix::scale(0.03, 0.03 * aspect);
        let digit_width = 0.025;
        let digit_height = 0.03 * aspect;

        renderer.draw_text(
            target,
            "0",
            text_scale * Matrix::translation(-0.81 - digit_width, -0.71 - digit_height),
            draw_parameters.clone(),
        );

        let text = format!(
            "{}",
            self.data
                .first()
                .map_or(1, |(_, stats)| stats.population + stats.dead),
        );
        renderer.draw_text(
            target,
            &text,
            text_scale
                * Matrix::translation(-0.81 - text.len() as f32 * digit_width, 0.99 - digit_height),
            draw_parameters.clone(),
        );

        self.draw_time_ticks(
            target,
            renderer,
            text_scale,
            digit_width,
            digit_height,
            draw_parameters,
        );
    }

    fn draw_time_ticks(
        &self,
        target: &mut Frame,
        renderer: &Renderer,
        text_scale: Matrix,
        digit_width: f32,
        digit_height: f32,
        draw_parameters: &DrawParameters,
    ) {
        let min_step = (self.max_t() / 8.0) as f32;
        let max_step = (self.max_t() / 3.0) as f32;

        let exp = min_step.log10().floor() as i32;
        let test_step = 10.0_f32.powi(exp);

        let step = if test_step >= min_step && test_step <= max_step {
            test_step
        } else if 2.0 * test_step >= min_step && 2.0 * test_step <= max_step {
            test_step * 2.0
        } else if 5.0 * test_step >= min_step && 5.0 * test_step <= max_step {
            test_step * 5.0
        } else if 10.0 * test_step >= min_step && 10.0 * test_step <= max_step {
            test_step * 10.0
        } else {
            panic!(
                "no good step found! min_step = {}, max_step = {}, test_step = {}",
                min_step, max_step, test_step
            );
        };

        let mut print_t = |t: f32| {
            let text = if step < 1.0 {
                format!("{:.1}", t)
            } else {
                format!("{:.0}", t)
            };
            let x = self.data_to_vertex(t as f64, 0, 1).position[0] as f32;
            renderer.draw_text(
                target,
                &text,
                text_scale
                    * Matrix::translation(
                        x - 0.01 - text.len() as f32 * digit_width,
                        -0.73 - digit_height,
                    ),
                draw_parameters.clone(),
            );
        };

        let mut t = step;
        while t < self.max_t() as f32 {
            print_t(t);
            t += step;
        }
        print_t(self.max_t() as f32);
    }

    fn generate_graph_vertices(&self) -> Vec<Vertex> {
        let mut result = vec![];

        for (t, stats) in &self.data {
            let total = stats.population + stats.dead;

            // vertices along the horizontal axis
            result.push(self.data_to_vertex(*t, 0, total));
            // vaccinated strip
            result.push(self.data_to_vertex(
                *t,
                stats.vaccinated - stats.vaccinated_infected,
                total,
            ));
            // vaccinated_infected strip
            result.push(self.data_to_vertex(*t, stats.vaccinated, total));
            // infected strip
            result.push(self.data_to_vertex(
                *t,
                stats.vaccinated + stats.infected - stats.vaccinated_infected,
                total,
            ));
            // healed strip
            result.push(self.data_to_vertex(
                *t,
                stats.vaccinated + stats.infected - stats.vaccinated_infected + stats.healed,
                total,
            ));
            // healthy strip
            result.push(self.data_to_vertex(*t, total - stats.dead, total));
            // dead strip
            result.push(self.data_to_vertex(*t, total, total));
        }

        result
    }

    fn generate_graph_indices(&self) -> [Vec<u16>; 6] {
        let mut indices = [vec![], vec![], vec![], vec![], vec![], vec![]];

        for i in 0..self.data.len() as u16 {
            for j in 0..6 {
                indices[j].push(7 * i + j as u16);
                indices[j].push(7 * i + j as u16 + 1);
            }
        }

        indices
    }
}
